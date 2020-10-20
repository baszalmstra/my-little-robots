use crate::{runner::async_runner::AsyncRunner, PlayerRunner};
use async_std::{
    io,
    io::BufReader,
    pin::Pin,
    task::{Context, JoinHandle, Poll},
};
use futures::{
    channel::{mpsc, oneshot},
    stream::IntoAsyncRead,
    AsyncRead, AsyncReadExt, AsyncWrite, SinkExt, TryStreamExt,
};
use mlr_api::{PlayerInput, PlayerMemory, PlayerOutput, RunnerError};
use std::{
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};
use wasi_common::virtfs::pipe::{ReadPipe, WritePipe};
use wasmtime::{Config, Engine, InterruptHandle, Linker, Module, OptLevel, Store};
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

pub struct WasiRunner {
    engine: Engine,
    module: Module,
}

impl WasiRunner {
    pub fn new(path_to_module: PathBuf) -> anyhow::Result<Self> {
        let mut config = Config::default();
        config
            .interruptable(true)
            .cache_config_load_default()?
            .cranelift_opt_level(OptLevel::Speed);

        let engine = Engine::new(&config);
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
        let (host_stdout, client_stdout) = wasi_stdout();
        let (host_stdin, client_stdin) = wasi_stdin();

        // Start the tick function
        let (interrupt_handle, handle) = self.start(client_stdin, client_stdout).await?;

        // Construct a runner that performs the communication with the process
        let mut runner = AsyncRunner::new(host_stdin, BufReader::new(host_stdout));

        // Time the process out if it doesnt return a value without a certain time
        let timeout = Duration::from_millis(10);
        let result = match async_std::future::timeout(timeout, runner.run(input)).await {
            Ok(result) => result,
            Err(_) => {
                interrupt_handle.interrupt();
                return Err(RunnerError::Timeout(timeout));
            }
        };

        drop(handle);

        result
    }
}

impl WasiRunner {
    /// Starts the runner on a separate thread. Receives the `stdin` and `stdout` streams which are
    /// used to communicate with the wasi "process". Returns a tuple containing an interrupt handle
    /// to cancel all pending WASI operations and a join handle that can be used to await the
    /// closure of the WASI process.
    async fn start<R: Read + Send + 'static, W: Write + Send + 'static>(
        &self,
        stdin: R,
        stdout: W,
    ) -> Result<(InterruptHandle, JoinHandle<Result<(), RunnerError>>), RunnerError> {
        let engine = self.engine.clone();
        let module = self.module.clone();
        let (tx, rx) = oneshot::channel();

        let handle = async_std::task::spawn_blocking(move || -> Result<(), RunnerError> {
            let store = Store::new(&engine);
            let mut linker = Linker::new(&store);

            let interrupt_handle = store.interrupt_handle().map_err(|e| {
                RunnerError::InitError(format!("unable to create interrupt handle: {}", e))
            })?;

            let wasi_ctx = WasiCtxBuilder::new()
                .stdout(WritePipe::new(stdout))
                .stdin(ReadPipe::new(stdin))
                .build()
                .map_err(|e| RunnerError::InitError(format!("error initializing wasi: {:?}", e)))?;

            let wasi = Wasi::new(&store, wasi_ctx);
            wasi.add_to_linker(&mut linker).map_err(|e| {
                RunnerError::InitError(format!("error adding wasi to linker: {}", e))
            })?;

            let instance = linker.instantiate(&module).map_err(|e| {
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

            // Send the interrupt handle back right before we call the function
            tx.send(interrupt_handle).map_err(|_| {
                RunnerError::InitError("unable to send interrupt back to main thread".to_string())
            })?;

            entrypoint().map_err(|e| {
                eprintln!("err: {}", e);
                RunnerError::InternalError
            })?;

            Ok(())
        });

        // Wait for the interrupt to be sent back
        let interrupt = rx
            .await
            .map_err(|_| RunnerError::InitError(format!("no interrupt handle was send")))?;
        Ok((interrupt, handle))
    }
}

fn wasi_stdin() -> (HostWasiStdin, ClientWasiStdin) {
    let (tx, rx) = mpsc::channel(8);
    (
        HostWasiStdin { inner: Some(tx) },
        ClientWasiStdin {
            inner: rx.into_async_read(),
        },
    )
}

fn wasi_stdout() -> (HostWasiStdout, ClientWasiStdout) {
    let (tx, rx) = mpsc::channel(8);
    (
        HostWasiStdout {
            inner: rx.into_async_read(),
        },
        ClientWasiStdout { inner: tx },
    )
}

/// An AsyncWrite type representing a wasi stdin stream.
pub struct HostWasiStdin {
    inner: Option<mpsc::Sender<io::Result<Vec<u8>>>>,
}

impl AsyncWrite for HostWasiStdin {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let tx = match &mut self.inner {
            Some(tx) => tx,
            None => {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::Other,
                    "called write after shutdown",
                )))
            }
        };
        tx.poll_ready(cx).map(|res| {
            let kind = io::ErrorKind::BrokenPipe; // ?
            res.map_err(|e| io::Error::new(kind, e))
                .and_then(|()| {
                    tx.try_send(Ok(buf.to_owned()))
                        .map_err(|e| io::Error::new(kind, e))
                })
                .map(|()| buf.len())
        })
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        self.inner.take().map(drop);
        Poll::Ready(Ok(()))
    }
}

pub struct ClientWasiStdin {
    inner: IntoAsyncRead<mpsc::Receiver<io::Result<Vec<u8>>>>,
}

impl Read for ClientWasiStdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        async_std::task::block_on(async { self.inner.read(buf).await })
    }
}

/// An AsyncRead type representing a wasi stdout stream.
#[pin_project::pin_project]
pub struct HostWasiStdout {
    #[pin]
    inner: IntoAsyncRead<mpsc::Receiver<io::Result<Vec<u8>>>>,
}

pub struct ClientWasiStdout {
    inner: mpsc::Sender<io::Result<Vec<u8>>>,
}

impl AsyncRead for HostWasiStdout {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl Write for ClientWasiStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        async_std::task::block_on(async move {
            self.inner
                .send(Ok(buf.to_vec()))
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
            Ok(buf.len())
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
