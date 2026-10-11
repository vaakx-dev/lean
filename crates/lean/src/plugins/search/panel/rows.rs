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
            if !self.collapsed.contains(&file_index) {
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
        if !self.collapsed.remove(&file_index) {
            self.collapsed.insert(file_index);
        }
        self.rebuild_rows();
        cx.notify();
    }

    pub(super) fn all_collapsed(&self) -> bool {
        !self.results.files.is_empty() && self.collapsed.len() == self.results.files.len()
    }

    pub(super) fn toggle_all(&mut self, cx: &mut Context<Self>) {
        if self.all_collapsed() {
            self.collapsed.clear();
        } else {
            self.collapsed = (0..self.results.files.len()).collect();
        }
        self.rebuild_rows();
        cx.notify();
    }
}

pub(super) fn clickable_row(index: usize, theme: &Theme) -> Stateful<Div> {
    clickable_list_row(("search-row", index), ROW_HEIGHT, theme)
}
