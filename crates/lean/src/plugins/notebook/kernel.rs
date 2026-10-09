mod connection;
mod replies;
mod runtime;
mod stderr;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use jupyter_protocol::{ExecuteRequest, JupyterMessage};
use serde_json::Value;
use tokio::task::JoinHandle;

pub(super) struct KernelSpec {
    pub(super) folder: PathBuf,
    pub(super) environment: HashMap<String, String>,
}

pub(super) enum KernelEvent {
    Ready,
    Reply { request: String, reply: Reply },
    Stopped(Option<String>),
}

pub(super) enum Reply {
    Output(Value),
    Clear { wait: bool },
    ExecutionCount(u64),
    Idle,
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
        let runtime = runtime::runtime()?;
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
