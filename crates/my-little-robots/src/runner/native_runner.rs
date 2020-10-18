use crate::runner::async_runner::AsyncRunner;
use crate::PlayerRunner;
use async_process::{Command, Stdio};
use async_std::io::{BufReader, BufWriter};
use mlr_api::{PlayerInput, PlayerOutput, RunnerError};
use std::ffi::{OsStr, OsString};
use std::time::Duration;

pub struct CommandRunner {
    command: OsString,
    args: Vec<OsString>,
}

impl CommandRunner {
    pub fn new(
        command: impl AsRef<OsStr>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> CommandRunner {
        CommandRunner {
            command: command.as_ref().into(),
            args: args.into_iter().map(|arg| arg.as_ref().into()).collect(),
        }
    }
}

#[async_trait::async_trait]
impl PlayerRunner for CommandRunner {
    async fn run(&mut self, input: PlayerInput) -> Result<PlayerOutput, RunnerError> {
        let mut proc = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = BufWriter::new(proc.stdin.take().unwrap());
        let stdout = BufReader::new(proc.stdout.take().unwrap());

        // Construct a runner that performs the communication with the process
        let mut runner = AsyncRunner::new(stdin, stdout);

        // Time the process out if it doesnt return a value without a certain time
        let timeout = Duration::from_millis(500);
        let result = async_std::future::timeout(timeout, runner.run(input))
            .await
            .map_err(|_| RunnerError::Timeout(timeout))?;

        // Kill the process if it doesnt quit in time
        if async_std::future::timeout(Duration::from_millis(1), proc.status())
            .await
            .is_err()
        {
            let _err = proc.kill();
        }

        result
    }
}
