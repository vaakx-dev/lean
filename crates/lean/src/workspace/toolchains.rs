use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::Workspace;
use crate::terminal::{TerminalLaunch, shell_kind};
use crate::toolchain::ToolchainsEvent;

impl Workspace {
    pub(super) fn terminal_launch(&self, cx: &App) -> TerminalLaunch {
        self.context
            .toolchains
            .read(cx)
            .terminal_launch(shell_kind())
    }

    pub(super) fn handle_toolchains_event(
        &mut self,
        event: &ToolchainsEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sync_terminal_launch(cx);
        if let ToolchainsEvent::Selected(label) = event
            && self.terminal.is_some()
        {
            window.push_notification(
                format!("New terminals will use {label}. Open a new terminal to switch."),
                cx,
            );
        }
        cx.notify();
    }
}
