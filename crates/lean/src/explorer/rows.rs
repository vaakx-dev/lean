use std::ops::Range;
use std::path::PathBuf;

use gpui_kit::component::{ActiveTheme as _, Icon, Theme, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{Explorer, ExplorerEvent, VisibleEntry, chevron_icon};
use crate::files::file_icon;
use crate::project::Project;

const ROW_HEIGHT: Pixels = px(26.);
const INDENT: Pixels = px(10.);

impl Explorer {
    pub(super) fn render_rows(
        &mut self,
        range: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let is_focused = self.focus_handle.contains_focused(window, cx);
        let mut rows = Vec::with_capacity(range.len());
        for index in range {
            if let Some(entry) = self.visible.get(index) {
                rows.push(self.render_row(index, entry, is_focused, cx));
            }
        }
        rows
    }

    fn render_row(
        &self,
        index: usize,
        entry: &VisibleEntry,
        is_focused: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let project = self.project.read(cx);
        let is_selected = self.selected.as_ref() == Some(&entry.path);
        let path = entry.path.clone();
        let is_dir = entry.is_dir;
        h_flex()
            .id(index)
            .h(ROW_HEIGHT)
            .w_full()
            .pl(px(12.) + INDENT * entry.depth as f32)
            .pr_3()
            .gap_1p5()
            .rounded(theme.radius)
            .border_1()
            .border_color(transparent_black())
            .cursor_pointer()
            .hover(|style| style.bg(theme.list_hover))
            .when(is_selected, |row| {
                row.bg(theme.list_active)
                    .when(is_focused, |row| row.border_color(theme.list_active_border))
            })
            .child(self.leading_icon(entry, theme))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(entry.name.clone()),
            )
            .child(
                h_flex()
                    .w_4()
                    .flex_shrink_0()
                    .justify_center()
                    .child(status_marker(entry, project, theme)),
            )
            .on_click(cx.listener(move |explorer, _, window, cx| {
                explorer.click_entry(path.clone(), is_dir, window, cx)
            }))
            .into_any_element()
    }

    fn leading_icon(&self, entry: &VisibleEntry, theme: &Theme) -> Icon {
        if entry.is_dir {
            Icon::empty()
                .path(chevron_icon(self.expanded.contains(&entry.path)))
                .size_4()
                .text_color(theme.muted_foreground)
        } else {
            file_icon(&entry.name)
        }
    }

    fn click_entry(
        &mut self,
        path: PathBuf,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        self.selected = Some(path.clone());
        if is_dir {
            self.toggle_folder(path, cx);
        } else {
            cx.emit(ExplorerEvent::OpenFile(path));
            cx.notify();
        }
    }
}

fn status_marker(entry: &VisibleEntry, project: &Project, theme: &Theme) -> AnyElement {
    if let Some(status) = project.status_for(&entry.path) {
        status.marker(theme).into_any_element()
    } else if entry.is_dir && project.folder_has_changes(&entry.path) {
        div()
            .size(px(5.))
            .rounded_full()
            .bg(theme.muted_foreground)
            .into_any_element()
    } else {
        div().into_any_element()
    }
}
