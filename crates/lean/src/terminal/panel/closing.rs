use gpui_kit::*;

use super::{TerminalPanel, TerminalPanelEvent};
use crate::terminal::settings::CONFIRM_CLOSE_RUNNING;
use crate::terminal::view::TerminalView;

const CLOSE_ANSWER: usize = 0;

impl TerminalPanel {
    pub(super) fn close(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.tabs.get(index).map(|tab| tab.view.clone()) else {
            return;
        };
        if CONFIRM_CLOSE_RUNNING.get(cx) && view.read(cx).is_busy() {
            self.confirm_close(view, window, cx);
        } else {
            self.remove(&view, window, cx);
        }
    }

    fn confirm_close(
        &mut self,
        view: Entity<TerminalView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            "Close this terminal?",
            Some("A command is still running in it and will be stopped."),
            &["Close", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |panel, cx| {
            if !matches!(answer.await, Ok(CLOSE_ANSWER)) {
                return;
            }
            panel
                .update_in(cx, |panel, window, cx| panel.remove(&view, window, cx))
                .ok();
        })
        .detach();
    }

    fn remove(&mut self, view: &Entity<TerminalView>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.tabs.iter().position(|tab| tab.view == *view) else {
            return;
        };
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active = 0;
            cx.emit(TerminalPanelEvent::Hide);
        } else {
            if self.active > index || self.active >= self.tabs.len() {
                self.active = self.active.saturating_sub(1);
            }
            self.focus_active(window, cx);
        }
        cx.notify();
    }
}
