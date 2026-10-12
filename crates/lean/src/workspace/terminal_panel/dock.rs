use std::path::Path;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::resize::{self, TerminalResize, panel_height, terminal_resize_handle};
use crate::terminal::settings::{CONFIRM_CLOSE_RUNNING, PANEL_HEIGHT};
use crate::terminal::{TerminalLaunch, TerminalPanel, TerminalPanelEvent, shell_kind};
use crate::workspace::Workspace;

pub(in crate::workspace) struct TerminalDock {
    panel: Entity<TerminalPanel>,
    height: Pixels,
    _save_height: Task<()>,
    _subscriptions: [Subscription; 2],
}

impl Workspace {
    pub(in crate::workspace) fn show_terminal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal_visible = true;
        let panel = self.terminal_panel(window, cx);
        panel.update(cx, |panel, cx| panel.reveal(window, cx));
        cx.notify();
    }

    pub(in crate::workspace) fn hide_terminal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal_visible = false;
        let terminal_had_focus = self.terminal.as_ref().is_some_and(|dock| {
            let panel = dock.panel.read(cx);
            panel.is_empty() || panel.contains_focus(window, cx)
        });
        if terminal_had_focus {
            self.context
                .editor_area
                .update(cx, |area, cx| area.focus_active(window, cx));
        }
        cx.notify();
    }

    pub(in crate::workspace) fn reset_terminal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.terminal = None;
        if self.terminal_visible {
            self.terminal_panel(window, cx);
        }
    }

    pub(in crate::workspace) fn running_terminal_count(&self, cx: &App) -> usize {
        if !CONFIRM_CLOSE_RUNNING.get(cx) {
            return 0;
        }
        self.terminal
            .as_ref()
            .map_or(0, |dock| dock.panel.read(cx).busy_count(cx))
    }

    pub(in crate::workspace) fn render_terminal_panel(
        &self,
        cx: &Context<Self>,
    ) -> Option<Stateful<Div>> {
        if !self.terminal_visible {
            return None;
        }
        let dock = self.terminal.as_ref()?;
        let theme = cx.theme();
        Some(
            div()
                .id("terminal-dock")
                .relative()
                .h(dock.height)
                .flex_shrink_0()
                .on_drag_move(cx.listener(Self::resize_terminal))
                .child(
                    div()
                        .size_full()
                        .overflow_hidden()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.sidebar)
                        .child(
                            dock.panel
                                .clone()
                                .cached(StyleRefinement::default().size_full()),
                        ),
                )
                .child(terminal_resize_handle()),
        )
    }

    fn resize_terminal(
        &mut self,
        event: &DragMoveEvent<TerminalResize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(dock) = self.terminal.as_mut() else {
            return;
        };
        let requested = event.bounds.bottom() - event.event.position.y;
        let height = panel_height(requested, window.viewport_size().height);
        if height == dock.height {
            return;
        }
        dock.height = height;
        dock._save_height = cx.spawn(async move |_, cx| {
            cx.background_executor().timer(resize::SAVE_DELAY).await;
            cx.update(|cx| PANEL_HEIGHT.set(height.as_f32().round(), cx));
        });
        cx.notify();
    }

    fn terminal_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TerminalPanel> {
        if let Some(dock) = &self.terminal {
            return dock.panel.clone();
        }
        let editor_area = self.context.editor_area.downgrade();
        let toolchains = self.context.toolchains.downgrade();
        self.context
            .toolchains
            .update(cx, |toolchains, cx| toolchains.ensure_discovered(cx));
        let launch = move |cx: &App| TerminalLaunch {
            active_file: editor_area
                .upgrade()
                .and_then(|area| area.read(cx).active_path().map(Path::to_path_buf)),
            ..toolchains
                .upgrade()
                .map(|toolchains| toolchains.read(cx).terminal_launch(shell_kind(cx), cx))
                .unwrap_or_default()
        };
        let root = self.project_root(cx);
        let panel = cx.new(|cx| TerminalPanel::new(root, launch, cx));
        let events = cx.subscribe_in(
            &panel,
            window,
            |workspace, _, event: &TerminalPanelEvent, window, cx| match event {
                TerminalPanelEvent::Hide => workspace.hide_terminal(window, cx),
            },
        );
        let height_setting =
            PANEL_HEIGHT.observe_in(window, cx, |workspace, height, window, cx| {
                if let Some(dock) = workspace.terminal.as_mut() {
                    dock.height = panel_height(px(height), window.viewport_size().height);
                    cx.notify();
                }
            });
        let height = panel_height(px(PANEL_HEIGHT.get(cx)), window.viewport_size().height);
        self.terminal = Some(TerminalDock {
            panel: panel.clone(),
            height,
            _save_height: Task::ready(()),
            _subscriptions: [events, height_setting],
        });
        panel
    }
}
