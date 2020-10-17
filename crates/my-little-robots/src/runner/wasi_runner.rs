use crate::PlayerRunner;
use async_std::io::{BufReader, BufWriter};
use mlr_api::{PlayerInput, PlayerMemory, PlayerOutput, RunnerError};
use std::io::{BufRead, Cursor};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
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

        let mut output = Arc::new(RwLock::new(Vec::<u8>::new()));

        {
            let wasi_ctx = WasiCtxBuilder::new()
                .inherit_stdout()
                .stdout(WritePipe::from_shared(output.clone()))
                .stdin(ReadPipe::from(input_json))
                .build()
                .map_err(|_| RunnerError::InternalError)?;

            let wasi = Wasi::new(&store, wasi_ctx);
            wasi.add_to_linker(&mut linker)
                .map_err(|_| RunnerError::InternalError)?;
            linker
                .module("", &self.module)
                .map_err(|_| RunnerError::InternalError)?;
            let default_export = linker
                .get_default("")
                .map_err(|_| RunnerError::InternalError)?;
            let entrypoint = default_export
                .get0::<()>()
                .map_err(|_| RunnerError::InternalError)?;
            let result = entrypoint().map_err(|e| {
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

        Err(RunnerError::InternalError)
    }
}
