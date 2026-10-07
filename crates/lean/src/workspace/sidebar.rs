use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::*;

use super::Workspace;
use crate::placeholder::placeholder;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarView {
    Explorer,
    Search,
    SourceControl,
}

impl SidebarView {
    pub(super) fn title(self) -> &'static str {
        match self {
            SidebarView::Explorer => "Explorer",
            SidebarView::Search => "Search",
            SidebarView::SourceControl => "Source Control",
        }
    }
}

impl Workspace {
    pub(super) fn show_view(
        &mut self,
        view: SidebarView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_view = view;
        self.sidebar_visible = true;
        self.focus_sidebar(window, cx);
        cx.notify();
    }

    pub(super) fn toggle_view(
        &mut self,
        view: SidebarView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_visible && self.sidebar_view == view {
            self.hide_sidebar(window, cx);
        } else {
            self.show_view(view, window, cx);
        }
    }

    pub(super) fn hide_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let sidebar_has_focus = self
            .sidebar_focus_handle(cx)
            .is_some_and(|focus_handle| focus_handle.contains_focused(window, cx));
        self.sidebar_visible = false;
        if sidebar_has_focus {
            self.editor_area
                .update(cx, |area, cx| area.focus_active(window, cx));
        }
        cx.notify();
    }

    pub(crate) fn focus_sidebar(&self, window: &mut Window, cx: &mut App) {
        if let Some(focus_handle) = self.sidebar_focus_handle(cx) {
            window.focus(&focus_handle, cx);
        }
    }

    fn sidebar_focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        match self.sidebar_view {
            SidebarView::Explorer => Some(self.explorer.focus_handle(cx)),
            SidebarView::Search => None,
            SidebarView::SourceControl => Some(self.source_control.focus_handle(cx)),
        }
    }

    pub(super) fn render_sidebar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let content = match self.sidebar_view {
            SidebarView::Explorer => self.explorer.clone().into_any_element(),
            SidebarView::Search => placeholder("Find in files is coming soon.", cx),
            SidebarView::SourceControl => self.source_control.clone().into_any_element(),
        };
        v_flex()
            .w(self.sidebar_width)
            .flex_shrink_0()
            .h_full()
            .overflow_hidden()
            .border_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                h_flex()
                    .h(px(36.))
                    .flex_shrink_0()
                    .px_3()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.sidebar_view.title()),
            )
            .child(div().flex_1().min_h_0().child(content))
    }
}
