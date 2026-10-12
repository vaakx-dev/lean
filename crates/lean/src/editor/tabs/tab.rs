use gpui_kit::component::{ActiveTheme as _, Icon, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Tabs;
use crate::editor::EditorArea;
use crate::editor::open_file::{OpenFile, TabKind};
use crate::files::file_icon;

const TAB_GROUP: &str = "tab";

#[derive(PartialEq)]
pub(super) struct TabState {
    name: SharedString,
    view_icon: Option<SharedString>,
    active: bool,
    dirty: bool,
    deleted: bool,
    preview: bool,
    conflict: bool,
}

impl TabState {
    pub(super) fn collect(area: &EditorArea) -> Vec<Self> {
        area.files
            .iter()
            .enumerate()
            .map(|(index, file)| Self::of(file, area.active == Some(index)))
            .collect()
    }

    fn of(file: &OpenFile, active: bool) -> Self {
        let view_icon = match &file.kind {
            TabKind::File => None,
            TabKind::View { icon } => Some(icon.clone()),
        };
        Self {
            name: file.name.clone(),
            view_icon,
            active,
            dirty: file.dirty,
            deleted: file.deleted,
            preview: file.preview,
            conflict: file.disk.conflict,
        }
    }

    pub(super) fn render(&self, index: usize, cx: &Context<Tabs>) -> Stateful<Div> {
        let theme = cx.theme();
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
            .when(self.active, |tab| {
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
            .when(!self.active, |tab| {
                tab.hover(|style| style.bg(theme.list_hover))
            })
            .on_click(cx.listener(move |tabs, event: &ClickEvent, window, cx| {
                let pin = event.click_count() > 1;
                tabs.update_area(cx, |area, cx| {
                    if pin {
                        area.pin(index, cx);
                    }
                    area.activate(index, window, cx);
                });
            }))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |tabs, _, window, cx| {
                    tabs.update_area(cx, |area, cx| area.close(index, window, cx));
                }),
            )
            .child(self.icon())
            .child(
                div()
                    .when(self.deleted, |name| name.line_through())
                    .when(self.preview, |name| name.italic())
                    .child(self.name.clone()),
            )
            .when(self.conflict, |tab| {
                tab.child(
                    Icon::empty()
                        .path("icons/triangle-alert.svg")
                        .size(px(14.))
                        .text_color(theme.warning),
                )
            })
            .child(self.close_button(index, cx))
    }

    fn icon(&self) -> Icon {
        match &self.view_icon {
            None => file_icon(&self.name),
            Some(icon) => Icon::empty().path(icon.clone()).size_4(),
        }
    }

    fn close_button(&self, index: usize, cx: &Context<Tabs>) -> Stateful<Div> {
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
            .on_click(cx.listener(move |tabs, _, window, cx| {
                cx.stop_propagation();
                tabs.update_area(cx, |area, cx| area.close(index, window, cx));
            }))
            .child(
                div()
                    .flex()
                    .when(self.dirty || !self.active, |icon| icon.invisible())
                    .group_hover(TAB_GROUP, |style| style.visible())
                    .child(Icon::empty().path("icons/x.svg").size(px(14.))),
            )
            .when(self.dirty, |close| {
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
}
