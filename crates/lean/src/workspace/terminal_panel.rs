use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::Workspace;
use crate::terminal::{TerminalPanel, TerminalPanelEvent};

const TERMINAL_HEIGHT: Pixels = px(260.);

pub(super) struct TerminalDock {
    panel: Entity<TerminalPanel>,
    _subscription: Subscription,
}

impl Workspace {
    pub(super) fn show_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal_visible = true;
        let panel = self.terminal_panel(window, cx);
        panel.update(cx, |panel, cx| panel.focus_active(window, cx));
        cx.notify();
    }

    pub(super) fn hide_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal_visible = false;
        let terminal_has_focus = self
            .terminal
            .as_ref()
            .is_some_and(|dock| dock.panel.read(cx).contains_focus(window, cx));
        if terminal_has_focus {
            self.editor_area
                .update(cx, |area, cx| area.focus_active(window, cx));
        }
        cx.notify();
    }

    pub(super) fn reset_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.terminal = None;
        if self.terminal_visible {
            self.terminal_panel(window, cx);
        }
    }

    pub(super) fn render_terminal_panel(&self, cx: &App) -> Option<Div> {
        if !self.terminal_visible {
            return None;
        }
        let dock = self.terminal.as_ref()?;
        let theme = cx.theme();
        Some(
            div()
                .h(TERMINAL_HEIGHT)
                .flex_shrink_0()
                .overflow_hidden()
                .border_1()
                .border_color(theme.border)
                .bg(theme.sidebar)
                .child(dock.panel.clone()),
        )
    }

    fn terminal_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TerminalPanel> {
        if let Some(dock) = &self.terminal {
            return dock.panel.clone();
        }
        let root = self.project.read(cx).root().to_path_buf();
        let panel = cx.new(|cx| TerminalPanel::new(root, cx));
        let subscription = cx.subscribe_in(
            &panel,
            window,
            |workspace, _, event: &TerminalPanelEvent, window, cx| match event {
                TerminalPanelEvent::Hide => workspace.hide_terminal(window, cx),
            },
        );
        self.terminal = Some(TerminalDock {
            panel: panel.clone(),
            _subscription: subscription,
        });
        panel
    }
}
