use std::ops::Range;

use gpui_kit::component::{ActiveTheme as _, Icon, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::file_operations::InlineEdit;
use super::{Explorer, VisibleEntry, chevron_icon};
use crate::files::file_icon;
use crate::plugins::{FileDecoration, PanelEvent};

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
        let decoration = (!entry.is_draft)
            .then(|| self.decorations.read(cx).get(&entry.path))
            .flatten();
        let edit = self.edit_for(entry);
        let is_editing = edit.is_some();
        let is_selected =
            !is_editing && !entry.is_draft && self.selected.as_ref() == Some(&entry.path);
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
            .when(!is_editing, |row| {
                row.cursor_pointer()
                    .hover(|style| style.bg(theme.list_hover))
            })
            .when(is_selected, |row| {
                row.bg(theme.list_active)
                    .when(is_focused, |row| row.border_color(theme.list_active_border))
            })
            .child(self.leading_icon(entry, edit, cx))
            .child(match edit {
                Some(edit) => edit.render_field(theme).into_any_element(),
                None => div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .child(entry.name.clone())
                    .into_any_element(),
            })
            .child(
                h_flex()
                    .w_4()
                    .flex_shrink_0()
                    .justify_center()
                    .children(decoration.map(decoration_marker)),
            )
            .when(!is_editing, |row| {
                row.on_click(cx.listener(move |explorer, _, window, cx| {
                    explorer.click_entry(index, window, cx)
                }))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |explorer, _, _, cx| {
                        explorer.select_for_context_menu(index, cx)
                    }),
                )
            })
            .into_any_element()
    }

    fn leading_icon(&self, entry: &VisibleEntry, edit: Option<&InlineEdit>, cx: &App) -> Icon {
        if entry.is_dir {
            let is_expanded = !entry.is_draft && self.expanded.contains(&entry.path);
            Icon::empty()
                .path(chevron_icon(is_expanded))
                .size_4()
                .text_color(cx.theme().muted_foreground)
        } else if let Some(edit) = edit.filter(|_| entry.is_draft) {
            file_icon(&edit.draft_name(cx))
        } else {
            file_icon(&entry.name)
        }
    }

    fn select_for_context_menu(&mut self, index: usize, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if let Some(entry) = self.visible.get(index).filter(|entry| !entry.is_draft) {
            self.selected = Some(entry.path.clone());
            cx.notify();
        }
    }

    fn click_entry(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.visible.get(index).filter(|entry| !entry.is_draft) else {
            return;
        };
        let path = entry.path.clone();
        let is_dir = entry.is_dir;
        window.focus(&self.focus_handle, cx);
        self.selected = Some(path.clone());
        if is_dir {
            self.toggle_folder(path, cx);
        } else {
            cx.emit(PanelEvent::OpenFile(path));
            cx.notify();
        }
    }
}

fn decoration_marker(decoration: FileDecoration) -> Div {
    match decoration {
        FileDecoration::Letter { letter, color } => div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(color)
            .child(letter),
        FileDecoration::Dot(color) => div().size(px(5.)).rounded_full().bg(color),
    }
}
