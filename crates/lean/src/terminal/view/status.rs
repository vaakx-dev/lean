use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::TerminalView;

pub(super) enum TerminalStatus {
    Starting,
    Running,
    Exited(Option<i32>),
    Failed(SharedString),
}

impl TerminalView {
    pub(super) fn render_status(&self, cx: &App) -> Option<Div> {
        let message = status_message(&self.status)?;
        Some(
            div()
                .flex_shrink_0()
                .pt_1()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(message),
        )
    }
}

fn status_message(status: &TerminalStatus) -> Option<String> {
    match status {
        TerminalStatus::Starting => Some("Starting the shell...".to_owned()),
        TerminalStatus::Running => None,
        TerminalStatus::Exited(Some(code)) if *code != 0 => Some(format!(
            "Process exited with code {code}. Press Enter to restart."
        )),
        TerminalStatus::Exited(_) => Some("Process exited. Press Enter to restart.".to_owned()),
        TerminalStatus::Failed(error) => Some(format!(
            "Failed to start the shell: {error}. Press Enter to retry."
        )),
    }
}
