use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::Workspace;
use super::sidebar::SidebarView;
use crate::app::actions::{
    About, CloseActiveEditor, NextEditor, PreviousEditor, SaveActiveEditor, ShowExplorer,
    ShowSearch, ShowSourceControl, ToggleSidebar, ToggleTerminal,
};

pub(super) fn bind_commands(element: Stateful<Div>, cx: &Context<Workspace>) -> Stateful<Div> {
    element
        .on_action(cx.listener(Workspace::open_folder))
        .on_action(cx.listener(Workspace::save_active_editor))
        .on_action(cx.listener(Workspace::close_active_editor))
        .on_action(cx.listener(Workspace::next_editor))
        .on_action(cx.listener(Workspace::previous_editor))
        .on_action(cx.listener(Workspace::toggle_sidebar))
        .on_action(cx.listener(Workspace::toggle_terminal))
        .on_action(cx.listener(Workspace::show_explorer))
        .on_action(cx.listener(Workspace::show_search))
        .on_action(cx.listener(Workspace::show_source_control))
        .on_action(cx.listener(Workspace::about))
}

impl Workspace {
    fn save_active_editor(
        &mut self,
        _: &SaveActiveEditor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_area
            .update(cx, |area, cx| area.save_active(window, cx));
    }

    fn close_active_editor(
        &mut self,
        _: &CloseActiveEditor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor_area
            .update(cx, |area, cx| area.close_active(window, cx));
    }

    fn next_editor(&mut self, _: &NextEditor, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_area
            .update(cx, |area, cx| area.activate_next(window, cx));
    }

    fn previous_editor(&mut self, _: &PreviousEditor, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_area
            .update(cx, |area, cx| area.activate_previous(window, cx));
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, window: &mut Window, cx: &mut Context<Self>) {
        if self.sidebar_visible {
            self.hide_sidebar(window, cx);
        } else {
            self.sidebar_visible = true;
            cx.notify();
        }
    }

    fn toggle_terminal(&mut self, _: &ToggleTerminal, window: &mut Window, cx: &mut Context<Self>) {
        if self.terminal_visible {
            self.hide_terminal(window, cx);
        } else {
            self.show_terminal(window, cx);
        }
    }

    fn show_explorer(&mut self, _: &ShowExplorer, window: &mut Window, cx: &mut Context<Self>) {
        self.show_view(SidebarView::Explorer, window, cx);
    }

    fn show_search(&mut self, _: &ShowSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.show_view(SidebarView::Search, window, cx);
    }

    fn show_source_control(
        &mut self,
        _: &ShowSourceControl,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_view(SidebarView::SourceControl, window, cx);
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
