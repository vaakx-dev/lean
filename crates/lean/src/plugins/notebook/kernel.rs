mod connection;
mod replies;
mod stderr;

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use jupyter_protocol::{ExecuteRequest, JupyterMessage};
use serde_json::Value;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

#[cfg(windows)]
const DEFAULT_PYTHON: &str = "python";
#[cfg(not(windows))]
const DEFAULT_PYTHON: &str = "python3";

static RUNTIME: LazyLock<io::Result<Runtime>> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("lean-kernels")
        .enable_all()
        .build()
});

pub(super) struct KernelSpec {
    pub(super) folder: PathBuf,
    pub(super) environment: HashMap<String, String>,
    pub(super) program: String,
    pub(super) startup_timeout: Duration,
}

pub(super) fn kernel_program(interpreter: &str) -> String {
    match interpreter.trim() {
        "" => DEFAULT_PYTHON.to_string(),
        program => program.to_string(),
    }
}

pub(super) enum KernelEvent<O = Value> {
    Ready,
    Reply { request: String, reply: Reply<O> },
    Stopped(Option<String>),
}

pub(super) enum Reply<O = Value> {
    Output(O),
    Clear { wait: bool },
    ExecutionCount(u64),
    Idle,
}

impl KernelEvent {
    pub(super) fn map_output<O>(self, decode: impl FnOnce(Value) -> O) -> KernelEvent<O> {
        match self {
            KernelEvent::Ready => KernelEvent::Ready,
            KernelEvent::Stopped(error) => KernelEvent::Stopped(error),
            KernelEvent::Reply { request, reply } => KernelEvent::Reply {
                request,
                reply: match reply {
                    Reply::Output(json) => Reply::Output(decode(json)),
                    Reply::Clear { wait } => Reply::Clear { wait },
                    Reply::ExecutionCount(count) => Reply::ExecutionCount(count),
                    Reply::Idle => Reply::Idle,
                },
            },
        }
    }
}

enum Command {
    Execute(JupyterMessage),
    Interrupt,
    Kill,
}

pub(super) struct Kernel {
    commands: UnboundedSender<Command>,
    task: JoinHandle<()>,
}

impl Kernel {
    pub(super) fn start(spec: KernelSpec) -> Result<(Kernel, UnboundedReceiver<KernelEvent>)> {
        let runtime = RUNTIME
            .as_ref()
            .map_err(|error| anyhow!("failed to start the kernel runtime: {error}"))?;
        let (commands, command_receiver) = unbounded();
        let (event_sender, events) = unbounded();
        let task = runtime.spawn(connection::run(spec, command_receiver, event_sender));
        Ok((Kernel { commands, task }, events))
    }

    pub(super) fn execute(&self, code: String) -> String {
        let message: JupyterMessage = ExecuteRequest::new(code).into();
        let request = message.header.msg_id.clone();
        self.send(Command::Execute(message));
        request
    }

    pub(super) fn interrupt(&self) {
        self.send(Command::Interrupt);
    }

    pub(super) async fn kill(self) {
        self.send(Command::Kill);
        if let Err(error) = self.task.await {
            log::debug!("the kernel task failed while stopping: {error}");
        }
    }

    fn send(&self, command: Command) {
        if let Err(error) = self.commands.unbounded_send(command) {
            log::warn!("the kernel is no longer running: {error}");
        }
    }
}
