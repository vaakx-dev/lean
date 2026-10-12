use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Sizable as _, WindowExt as _};
use gpui_kit::*;

use super::Workspace;
use crate::toolchain::ToolchainsEvent;

struct InterpreterChanged;

impl Workspace {
    pub(super) fn handle_toolchains_event(
        &mut self,
        event: &ToolchainsEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let ToolchainsEvent::Selected(label) = event
            && self.terminal.is_some()
        {
            window.push_notification(interpreter_notice(label, cx.weak_entity()), cx);
        }
    }

    fn restart_terminals(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_terminal(window, cx);
        cx.notify();
    }
}

fn interpreter_notice(label: &SharedString, workspace: WeakEntity<Workspace>) -> Notification {
    Notification::info(format!(
        "New terminals will use {label}. Open terminals keep the previous interpreter."
    ))
    .id::<InterpreterChanged>()
    .action(move |_, _, cx| {
        let workspace = workspace.clone();
        Button::new("restart-terminals")
            .small()
            .label("Restart terminals")
            .on_click(cx.listener(move |notice, _, window, cx| {
                workspace
                    .update(cx, |workspace, cx| workspace.restart_terminals(window, cx))
                    .ok();
                notice.dismiss(window, cx);
            }))
    })
}
