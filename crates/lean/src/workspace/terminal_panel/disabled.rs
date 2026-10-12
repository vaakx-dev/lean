use gpui_kit::{App, Context, Div, Stateful, Window};

use crate::workspace::Workspace;

pub(in crate::workspace) enum TerminalDock {}

impl Workspace {
    pub(in crate::workspace) fn show_terminal(&mut self, _: &mut Window, _: &mut Context<Self>) {}

    pub(in crate::workspace) fn hide_terminal(&mut self, _: &mut Window, _: &mut Context<Self>) {}

    pub(in crate::workspace) fn reset_terminal(&mut self, _: &mut Window, _: &mut Context<Self>) {}

    pub(in crate::workspace) fn running_terminal_count(&self, _: &App) -> usize {
        0
    }

    pub(in crate::workspace) fn render_terminal_panel(
        &self,
        _: &Context<Self>,
    ) -> Option<Stateful<Div>> {
        None
    }
}
