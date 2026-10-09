use gpui_kit::component::{ActiveTheme as _, Icon, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::EditorArea;
use super::open_file::{OpenFile, TabKind};
use crate::files::file_icon;

const TAB_GROUP: &str = "tab";

impl EditorArea {
    pub(super) fn render_tab_bar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .h(px(36.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(theme.border)
            .child(self.render_tabs(cx))
            .children(self.render_file_actions(cx))
    }

    fn render_tabs(&self, cx: &Context<Self>) -> Stateful<Div> {
        h_flex()
            .id("editor-tabs")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_x_scroll()
            .children(
                self.files
                    .iter()
                    .enumerate()
                    .map(|(index, file)| self.render_tab(index, file, cx)),
            )
    }

    fn render_tab(&self, index: usize, file: &OpenFile, cx: &Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        let is_active = self.active == Some(index);
        h_flex()
            .id(index)
            .group(TAB_GROUP)
            .relative()
            .h_full()
            .flex_shrink_0()
            .pl_3()
            .pr_1p5()
            .gap_1p5()
            .border_r_1()
            .border_color(theme.border)
            .cursor_pointer()
            .text_color(theme.muted_foreground)
            .when(is_active, |tab| {
                tab.bg(theme.background).text_color(theme.foreground).child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(px(1.))
                        .bg(theme.primary),
                )
            })
            .when(!is_active, |tab| {
                tab.hover(|style| style.bg(theme.list_hover))
            })
            .on_click(cx.listener(move |area, _, window, cx| area.activate(index, window, cx)))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |area, _, window, cx| area.close(index, window, cx)),
            )
            .child(tab_icon(file))
            .child(
                div()
                    .when(file.deleted, |name| name.line_through())
                    .child(file.name.clone()),
            )
            .when(file.disk.conflict, |tab| {
                tab.child(
                    Icon::empty()
                        .path("icons/triangle-alert.svg")
                        .size(px(14.))
                        .text_color(theme.warning),
                )
            })
            .child(close_button(index, file.dirty, is_active, cx))
    }
}

fn tab_icon(file: &OpenFile) -> Icon {
    match &file.kind {
        TabKind::File => file_icon(&file.name),
        TabKind::View { icon } => Icon::empty().path(icon.clone()).size_4(),
    }
}

fn close_button(
    index: usize,
    is_dirty: bool,
    is_active: bool,
    cx: &Context<EditorArea>,
) -> Stateful<Div> {
    let theme = cx.theme();
    div()
        .id("close")
        .relative()
        .size(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(theme.radius)
        .hover(|style| style.bg(theme.accent))
        .on_click(cx.listener(move |area, _, window, cx| {
            cx.stop_propagation();
            area.close(index, window, cx)
        }))
        .child(
            div()
                .flex()
                .when(is_dirty || !is_active, |icon| icon.invisible())
                .group_hover(TAB_GROUP, |style| style.visible())
                .child(Icon::empty().path("icons/x.svg").size(px(14.))),
        )
        .when(is_dirty, |close| {
            close.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .group_hover(TAB_GROUP, |style| style.invisible())
                    .child(div().size(px(8.)).rounded_full().bg(theme.foreground)),
            )
        })
}
