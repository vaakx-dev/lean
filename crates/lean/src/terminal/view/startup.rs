use std::time::Duration;

use gpui_kit::{Context, Task};

use super::TerminalView;

const IDLE_DELAY: Duration = Duration::from_millis(300);
const MAX_DELAY: Duration = Duration::from_secs(4);

#[derive(Default)]
pub(super) struct StartupCommand {
    pending: Option<String>,
    _idle_task: Option<Task<()>>,
    _deadline_task: Option<Task<()>>,
}

impl TerminalView {
    pub(super) fn queue_startup_command(&mut self, cx: &mut Context<Self>) {
        let pending = self.launch.startup_command.clone();
        let deadline_task = pending
            .is_some()
            .then(|| Self::send_startup_command_after(MAX_DELAY, cx));
        self.startup = StartupCommand {
            pending,
            _idle_task: None,
            _deadline_task: deadline_task,
        };
    }

    pub(super) fn startup_output_arrived(&mut self, cx: &mut Context<Self>) {
        if self.startup.pending.is_some() {
            self.startup._idle_task = Some(Self::send_startup_command_after(IDLE_DELAY, cx));
        }
    }

    fn send_startup_command_after(delay: Duration, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(delay).await;
            view.update(cx, |view, _| view.send_startup_command()).ok();
        })
    }

    fn send_startup_command(&mut self) {
        let Some(command) = self.startup.pending.take() else {
            return;
        };
        if let Some(session) = self.session() {
            session.write(format!("{command}\r").into_bytes());
        }
    }
}
