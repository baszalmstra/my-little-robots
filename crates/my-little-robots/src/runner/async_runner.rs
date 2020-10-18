use crate::PlayerRunner;
use futures::stream::StreamExt;
use futures::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use mlr_api::{PlayerInput, PlayerMemory, PlayerOutput, RunnerError};

pub struct AsyncRunner<W: AsyncWrite, R: AsyncBufRead> {
    stdout: R,
    stdin: W,
}

impl<W: AsyncWrite + Unpin + Send, R: AsyncBufRead + Unpin + Send> AsyncRunner<W, R> {
    pub fn new(stdin: W, stdout: R) -> Self {
        Self { stdin, stdout }
    }
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Unpin + Send, R: AsyncBufRead + Unpin + Send> PlayerRunner
    for AsyncRunner<W, R>
{
    async fn run(
        &mut self,
        input: PlayerInput<PlayerMemory>,
    ) -> Result<PlayerOutput<PlayerMemory>, RunnerError> {
        let mut input_json = serde_json::to_vec(&input)?;
        input_json.push(b'\n');
        self.stdin.write(&input_json).await?;
        self.stdin.flush().await?;

        let mut lines = (&mut self.stdout).lines();
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
    }
}
