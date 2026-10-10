use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use super::panels::SidebarPanel;
use super::settings::DEFAULT_PANEL;
use crate::explorer;

impl Workspace {
    pub(super) fn show_panel(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.panel_index(id) {
            self.activate_panel(index, window, cx);
        }
    }

    pub(super) fn toggle_panel(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_visible && self.active_panel == index {
            self.hide_sidebar(window, cx);
        } else {
            self.activate_panel(index, window, cx);
        }
    }

    pub(super) fn panel_index(&self, id: &str) -> Option<usize> {
        self.panels.iter().position(|panel| panel.panel.id == id)
    }

    pub(super) fn default_panel_index(&self, cx: &App) -> usize {
        self.panel_index(&DEFAULT_PANEL.get(cx)).unwrap_or(0)
    }

    fn activate_panel(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.active_panel = index;
        self.show_sidebar(window, cx);
    }

    pub(super) fn show_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_visible = true;
        self.focus_sidebar(window, cx);
        cx.notify();
    }

    pub(super) fn hide_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let sidebar_has_focus = self
            .active_sidebar_panel()
            .is_some_and(|panel| panel.view.focus_handle(cx).contains_focused(window, cx));
        self.sidebar_visible = false;
        if sidebar_has_focus {
            self.context
                .editor_area
                .update(cx, |area, cx| area.focus_active(window, cx));
        }
        cx.notify();
    }

    pub(crate) fn focus_initial(&self, window: &mut Window, cx: &mut App) {
        if self.shows_sidebar() {
            self.focus_sidebar(window, cx);
        } else {
            self.context
                .editor_area
                .update(cx, |area, cx| area.focus_active(window, cx));
        }
    }

    pub(super) fn focus_sidebar(&self, window: &mut Window, cx: &mut App) {
        match self.active_sidebar_panel() {
            Some(panel) => panel.view.focus(window, cx),
            None => window.focus(&self.focus_handle, cx),
        }
    }

    pub(super) fn shows_sidebar(&self) -> bool {
        self.sidebar_visible && self.active_sidebar_panel().is_some()
    }

    fn active_sidebar_panel(&self) -> Option<&SidebarPanel> {
        self.panels.get(self.active_panel)
    }

    pub(super) fn render_sidebar(&self, width: Pixels, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let panel = self.active_sidebar_panel();
        v_flex()
            .w(width)
            .flex_shrink_0()
            .h_full()
            .overflow_hidden()
            .border_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .when_some(panel, |sidebar, panel| {
                sidebar
                    .when(panel.panel.id != explorer::PANEL.id, |sidebar| {
                        sidebar.child(
                            h_flex()
                                .h(px(36.))
                                .flex_shrink_0()
                                .px_3()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(panel.panel.title),
                        )
                    })
                    .child(
                        div().flex_1().min_h_0().child(
                            panel
                                .view
                                .view()
                                .cached(StyleRefinement::default().size_full()),
                        ),
                    )
            })
    }
}
