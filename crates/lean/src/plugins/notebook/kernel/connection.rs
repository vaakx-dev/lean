use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use jupyter_protocol::{
    ConnectionInfo, InterruptRequest, JupyterMessage, KernelInfoRequest, ShutdownRequest, Transport,
};
use runtimelib::{
    ClientControlConnection, ClientIoPubConnection, DealerRecvConnection, DealerSendConnection,
};
use tokio::process::{Child, Command as Process};
use uuid::Uuid;

use super::replies::kernel_event;
use super::stderr::KernelStderr;
use super::{Command, KernelEvent, KernelSpec};

#[cfg(windows)]
const PYTHON: &str = "python";
#[cfg(not(windows))]
const PYTHON: &str = "python3";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
const NUDGE_INTERVAL: Duration = Duration::from_millis(500);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

struct Sockets {
    iopub: ClientIoPubConnection,
    shell: DealerSendConnection,
    shell_replies: DealerRecvConnection,
    control: ClientControlConnection,
}

enum Step {
    Command(Option<Command>),
    Message(JupyterMessage),
    Nudge,
    Stderr(Option<String>),
    Exited(ExitStatus),
}

pub(super) async fn run(
    spec: KernelSpec,
    mut commands: UnboundedReceiver<Command>,
    events: UnboundedSender<KernelEvent>,
) {
    let error = serve(&spec, &mut commands, &events)
        .await
        .err()
        .map(|error| format!("{error:#}"));
    send(&events, KernelEvent::Stopped(error));
}

async fn serve(
    spec: &KernelSpec,
    commands: &mut UnboundedReceiver<Command>,
    events: &UnboundedSender<KernelEvent>,
) -> Result<()> {
    let info = connection_info().await?;
    let file = write_connection_file(&info).await?;
    let result = drive(spec, &info, &file, commands, events).await;
    if let Err(error) = tokio::fs::remove_file(&file).await {
        log::debug!("failed to remove {}: {error}", file.display());
    }
    result
}

async fn drive(
    spec: &KernelSpec,
    info: &ConnectionInfo,
    file: &Path,
    commands: &mut UnboundedReceiver<Command>,
    events: &UnboundedSender<KernelEvent>,
) -> Result<()> {
    let mut child = spawn_kernel(spec, file)?;
    let mut stderr = KernelStderr::new(child.stderr.take().context("the kernel has no stderr")?);
    let session = Uuid::new_v4().to_string();
    let mut sockets = tokio::select! {
        sockets = connect(info, &session) => sockets?,
        status = child.wait() => return Err(stderr.exit_error(status?).await),
    };
    let mut nudge = tokio::time::interval(NUDGE_INTERVAL);
    let startup = tokio::time::sleep(STARTUP_TIMEOUT);
    tokio::pin!(startup);
    let mut ready = false;
    loop {
        let step = tokio::select! {
            command = commands.next() => Step::Command(command),
            message = sockets.iopub.read() => match message {
                Ok(message) => Step::Message(message),
                Err(error) => {
                    log::warn!("unreadable kernel message: {error}");
                    continue;
                }
            },
            reply = sockets.shell_replies.read() => {
                if let Err(error) = reply {
                    log::warn!("unreadable kernel reply: {error}");
                }
                continue;
            },
            _ = nudge.tick(), if !ready => Step::Nudge,
            _ = &mut startup, if !ready => bail!("the kernel did not start within a minute"),
            line = stderr.next_line(), if stderr.is_open() => Step::Stderr(line?),
            status = child.wait() => Step::Exited(status?),
        };
        match step {
            Step::Command(Some(Command::Execute(message))) => sockets.shell.send(message).await?,
            Step::Command(Some(Command::Interrupt)) => {
                sockets.control.send(InterruptRequest {}.into()).await?
            }
            Step::Command(Some(Command::Kill)) => {
                child.start_kill()?;
                return Ok(());
            }
            Step::Command(None) => return shutdown(&mut sockets.control, &mut child).await,
            Step::Message(message) => {
                if !ready {
                    ready = true;
                    send(events, KernelEvent::Ready);
                }
                if let Some(event) = kernel_event(message) {
                    send(events, event);
                }
            }
            Step::Nudge => sockets.shell.send(KernelInfoRequest {}.into()).await?,
            Step::Stderr(line) => stderr.record(line),
            Step::Exited(status) => return Err(stderr.exit_error(status).await),
        }
    }
}

fn spawn_kernel(spec: &KernelSpec, connection_file: &Path) -> Result<Child> {
    let mut process = Process::new(PYTHON);
    process
        .args(["-m", "ipykernel_launcher", "-f"])
        .arg(connection_file)
        .current_dir(&spec.folder)
        .envs(&spec.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(not(windows))]
    process.env("JPY_PARENT_PID", std::process::id().to_string());
    #[cfg(windows)]
    process.creation_flags(CREATE_NO_WINDOW);
    process
        .spawn()
        .with_context(|| format!("failed to start {PYTHON}"))
}

async fn connect(info: &ConnectionInfo, session: &str) -> Result<Sockets> {
    let iopub = runtimelib::create_client_iopub_connection(info, "", session).await?;
    let identity = runtimelib::peer_identity_for_session(session)?;
    let (shell, shell_replies) =
        runtimelib::create_client_shell_connection_with_identity(info, session, identity)
            .await?
            .split();
    let control = runtimelib::create_client_control_connection(info, session).await?;
    Ok(Sockets {
        iopub,
        shell,
        shell_replies,
        control,
    })
}

async fn shutdown(control: &mut ClientControlConnection, child: &mut Child) -> Result<()> {
    control
        .send(ShutdownRequest { restart: false }.into())
        .await?;
    if tokio::time::timeout(SHUTDOWN_TIMEOUT, child.wait())
        .await
        .is_err()
    {
        child.kill().await?;
    }
    Ok(())
}

async fn connection_info() -> Result<ConnectionInfo> {
    let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let ports = runtimelib::peek_ports(ip, 5).await?;
    let &[stdin_port, control_port, hb_port, shell_port, iopub_port] = ports.as_slice() else {
        bail!("failed to reserve ports for the kernel");
    };
    Ok(ConnectionInfo {
        ip: ip.to_string(),
        transport: Transport::TCP,
        shell_port,
        iopub_port,
        stdin_port,
        control_port,
        hb_port,
        key: Uuid::new_v4().to_string(),
        signature_scheme: "hmac-sha256".into(),
        kernel_name: Some("python3".into()),
    })
}

async fn write_connection_file(info: &ConnectionInfo) -> Result<PathBuf> {
    let folder = runtimelib::dirs::runtime_dir();
    tokio::fs::create_dir_all(&folder)
        .await
        .with_context(|| format!("failed to create {}", folder.display()))?;
    let file = folder.join(format!("lean-kernel-{}.json", Uuid::new_v4()));
    tokio::fs::write(&file, serde_json::to_vec(info)?)
        .await
        .with_context(|| format!("failed to write {}", file.display()))?;
    Ok(file)
}

fn send(events: &UnboundedSender<KernelEvent>, event: KernelEvent) {
    if events.unbounded_send(event).is_err() {
        log::debug!("the notebook closed before its kernel stopped");
    }
}
