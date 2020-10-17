use crate::PlayerRunner;
use async_process::{Command, Stdio};
use async_std::io::{BufReader, BufWriter};
use futures::{AsyncBufReadExt, AsyncWriteExt, StreamExt};
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

        let mut stdin = BufWriter::new(proc.stdin.take().unwrap());
        let mut stdout = BufReader::new(proc.stdout.take().unwrap());

        let mut input_json = serde_json::to_vec(&input)?;
        input_json.push(b'\n');
        stdin.write(&input_json).await?;
        stdin.flush().await?;

        let mut lines = (&mut stdout).lines();
        let timeout = Duration::from_millis(10);
        let result = async_std::future::timeout(
            timeout,
            (|| async move {
                loop {
                    let line = lines
                        .next()
                        .await
                        .ok_or(RunnerError::NoData)?
                        .map_err(|_| RunnerError::NoData)?;
                    if let Some(output) = line.strip_prefix("__mlr_output:") {
                        return Ok(serde_json::from_str::<PlayerOutput>(output)?);
                    } else {
                        println!("Player {:?}: {}", input.player_id, line);
                    }
                }
            })(),
        )
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
