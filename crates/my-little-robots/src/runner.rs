mod native_runner;
mod wasi_runner;

use crate::runner::native_runner::CommandRunner;
use crate::runner::wasi_runner::WasiRunner;
use crate::PlayerRunner;
use mlr_api::{PlayerInput, PlayerOutput, RunnerError};
use std::ffi::OsStr;
use std::path::PathBuf;

/// A runner is something that can perform a player step
pub enum Runner {
    Command(CommandRunner),
    Wasi(WasiRunner),
}

impl Runner {
    pub fn new_cmd(
        command: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Runner {
        Runner::Command(CommandRunner::new(command, args))
    }

    pub fn new_wasm(path_to_module: PathBuf) -> anyhow::Result<Runner> {
        Ok(Runner::Wasi(WasiRunner::new(path_to_module)?))
    }
}

#[async_trait::async_trait]
impl PlayerRunner for Runner {
    async fn run(&mut self, input: PlayerInput) -> Result<PlayerOutput, RunnerError> {
        match self {
            Runner::Command(cmd) => cmd.run(input).await,
            Runner::Wasi(wasi) => wasi.run(input).await,
        }
    }
}
