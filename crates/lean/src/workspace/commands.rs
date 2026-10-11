use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::{
    About, CloseActiveEditor, NextEditor, OpenSettings, PreviousEditor, SaveActiveEditor,
    ShowPanel, ToggleSidebar, ToggleTerminal,
};
use crate::app::keymap::{GoToLine, RevealActiveFile};
use crate::{explorer, settings};

pub(super) fn bind_commands(element: Stateful<Div>, cx: &Context<Workspace>) -> Stateful<Div> {
    element
        .on_action(cx.listener(Workspace::go_to_line))
        .on_action(cx.listener(Workspace::reveal_active_file))
        .on_action(cx.listener(Workspace::open_folder))
        .on_action(cx.listener(Workspace::open_recent))
        .on_action(cx.listener(Workspace::open_recent_project))
        .on_action(cx.listener(Workspace::clear_recent_projects))
        .on_action(cx.listener(Workspace::save_active_editor))
        .on_action(cx.listener(Workspace::close_active_editor))
        .on_action(cx.listener(Workspace::next_editor))
        .on_action(cx.listener(Workspace::previous_editor))
        .on_action(cx.listener(Workspace::toggle_sidebar))
        .on_action(cx.listener(Workspace::toggle_terminal))
        .on_action(cx.listener(Workspace::show_panel_action))
        .on_action(cx.listener(Workspace::open_settings))
        .on_action(cx.listener(Workspace::about))
}

impl Workspace {
    fn save_active_editor(
        &mut self,
        _: &SaveActiveEditor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.save_active(window, cx));
    }

    fn close_active_editor(
        &mut self,
        _: &CloseActiveEditor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.close_active(window, cx));
    }

    fn next_editor(&mut self, _: &NextEditor, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.activate_next(window, cx));
    }

    fn previous_editor(&mut self, _: &PreviousEditor, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.activate_previous(window, cx));
    }

    fn go_to_line(&mut self, _: &GoToLine, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.prompt_go_to_line(window, cx));
    }

    fn reveal_active_file(
        &mut self,
        _: &RevealActiveFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if explorer::reveal_active_file(&self.context, cx) {
            self.show_panel(explorer::PANEL.id, window, cx);
        }
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, window: &mut Window, cx: &mut Context<Self>) {
        if self.sidebar_visible {
            self.hide_sidebar(window, cx);
        } else {
            self.show_sidebar(window, cx);
        }
    }

    fn toggle_terminal(&mut self, _: &ToggleTerminal, window: &mut Window, cx: &mut Context<Self>) {
        if self.terminal_visible {
            self.hide_terminal(window, cx);
        } else {
            self.show_terminal(window, cx);
        }
    }

    fn show_panel_action(
        &mut self,
        action: &ShowPanel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_panel(action.id, window, cx);
    }

    pub(super) fn open_settings(
        &mut self,
        _: &OpenSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        settings::open_settings(&self.context.editor_area, window, cx);
    }

    fn about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        window.push_notification(
            format!(
                "Lean {} — a small code editor built with GPUI.",
                env!("CARGO_PKG_VERSION")
            ),
            cx,
        );
    }
}
