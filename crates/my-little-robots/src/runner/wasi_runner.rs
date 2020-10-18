use crate::PlayerRunner;
use mlr_api::{PlayerInput, PlayerMemory, PlayerOutput, RunnerError};
use std::io::BufRead;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use wasi_common::virtfs::pipe::{ReadPipe, WritePipe};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

pub struct WasiRunner {
    engine: Engine,
    module: Module,
}

impl WasiRunner {
    pub fn new(path_to_module: PathBuf) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, &path_to_module)?;
        Ok(WasiRunner { engine, module })
    }
}

#[async_trait::async_trait]
impl PlayerRunner for WasiRunner {
    async fn run(
        &mut self,
        input: PlayerInput<PlayerMemory>,
    ) -> Result<PlayerOutput<PlayerMemory>, RunnerError> {
        let store = Store::new(&self.engine);
        let mut linker = Linker::new(&store);

        let mut input_json = serde_json::to_vec(&input)?;
        input_json.push(b'\n');

        let output = Arc::new(RwLock::new(Vec::<u8>::new()));
        {
            let wasi_ctx = WasiCtxBuilder::new()
                .stdout(WritePipe::from_shared(output.clone()))
                .stdin(ReadPipe::from(input_json))
                .build()
                .map_err(|e| RunnerError::InitError(format!("error initializing wasi: {:?}", e)))?;

            let wasi = Wasi::new(&store, wasi_ctx);
            wasi.add_to_linker(&mut linker).map_err(|e| {
                RunnerError::InitError(format!("error adding wasi to linker: {}", e))
            })?;

            let instance = linker.instantiate(&self.module).map_err(|e| {
                RunnerError::InitError(format!("error instantiating wasm module: {}", e))
            })?;
            let default_export = instance.get_func("_start").ok_or_else(|| {
                RunnerError::InitError(
                    "could not locate _start function in wasm module".to_string(),
                )
            })?;

            let entrypoint = default_export.get0::<()>().map_err(|e| {
                RunnerError::InitError(format!("error executing wasm module: {}", e))
            })?;
            entrypoint().map_err(|e| {
                eprintln!("err: {}", e);
                RunnerError::InternalError
            })?;
        }

        let output = output.read().unwrap();
        let mut lines = output.deref().lines();
        loop {
            let line = lines
                .next()
                .ok_or(RunnerError::NoData)?
                .map_err(|_| RunnerError::NoData)?;
            if let Some(output) = line.strip_prefix("__mlr_output:") {
                return Ok(serde_json::from_str::<PlayerOutput>(output)?);
            } else {
                println!("Player {:?}: {}", input.player_id, line);
            }
        }
    }
}
