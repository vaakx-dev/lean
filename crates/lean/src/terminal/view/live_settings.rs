use gpui_kit::{Context, Subscription};

use super::TerminalView;
use crate::settings::Settings;
use crate::terminal::session::term_config;

impl TerminalView {
    pub(super) fn observe_settings(cx: &mut Context<Self>) -> Subscription {
        cx.observe_global::<Settings>(|view, cx| view.apply_settings(cx))
    }

    fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let config = term_config(cx);
        if config != self.term_config {
            if let Some(session) = &self.session {
                session.set_config(config.clone());
            }
            self.term_config = config;
        }
        cx.notify();
    }
}
