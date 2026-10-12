use std::ops::Range;

use gpui_kit::component::Theme;
use gpui_kit::*;

use super::SearchPanel;
use crate::ui::clickable_list_row;

const ROW_HEIGHT: Pixels = px(22.);

#[derive(Clone, Copy)]
pub(super) enum Row {
    File(usize),
    Line(usize, usize),
}

impl SearchPanel {
    pub(super) fn rebuild_rows(&mut self) {
        self.rows.clear();
        for (file_index, file) in self.results.files.iter().enumerate() {
            self.rows.push(Row::File(file_index));
            if !file.collapsed {
                self.rows.extend(
                    (0..file.lines.len()).map(|line_index| Row::Line(file_index, line_index)),
                );
            }
        }
    }

    pub(super) fn render_rows(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        range
            .filter_map(|index| match *self.rows.get(index)? {
                Row::File(file_index) => self.render_file_row(index, file_index, cx),
                Row::Line(file_index, line_index) => {
                    self.render_line_row(index, file_index, line_index, cx)
                }
            })
            .collect()
    }

    pub(super) fn toggle_file(&mut self, file_index: usize, cx: &mut Context<Self>) {
        if let Some(file) = self.results.files.get_mut(file_index) {
            file.collapsed = !file.collapsed;
        }
        self.rebuild_rows();
        cx.notify();
    }

    pub(super) fn all_collapsed(&self) -> bool {
        !self.results.files.is_empty() && self.results.files.iter().all(|file| file.collapsed)
    }

    pub(super) fn toggle_all(&mut self, cx: &mut Context<Self>) {
        let collapsed = !self.all_collapsed();
        for file in &mut self.results.files {
            file.collapsed = collapsed;
        }
        self.rebuild_rows();
        cx.notify();
    }
}

pub(super) fn clickable_row(index: usize, theme: &Theme) -> Stateful<Div> {
    clickable_list_row(("search-row", index), ROW_HEIGHT, theme)
}
