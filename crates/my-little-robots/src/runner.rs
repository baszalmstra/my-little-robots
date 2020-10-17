mod native_runner;

use crate::runner::native_runner::CommandRunner;
use crate::PlayerRunner;
use mlr_api::{PlayerInput, PlayerOutput, RunnerError};
use std::ffi::OsStr;

/// A runner is something that can perform a player step
pub enum Runner {
    Command(CommandRunner),
}

impl Runner {
    pub fn new_cmd(
        command: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Runner {
        Runner::Command(CommandRunner::new(command, args))
    }
}

#[async_trait::async_trait]
impl PlayerRunner for Runner {
    async fn run(&mut self, input: PlayerInput) -> Result<PlayerOutput, RunnerError> {
        match self {
            Runner::Command(cmd) => cmd.run(input).await,
        }
    }
}
