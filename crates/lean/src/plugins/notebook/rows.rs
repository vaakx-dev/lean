use std::ops::Range;

use gpui_kit::*;

use super::Notebook;
use super::cell::CellId;

impl Notebook {
    pub(super) fn splice_rows(&self, old: Range<usize>, new: Range<usize>, cx: &App) {
        let focus_handles: Vec<Option<FocusHandle>> = new
            .map(|index| {
                self.cells
                    .get(index)
                    .map(|cell| cell.editor.focus_handle(cx))
            })
            .collect();
        self.list_state.splice_focusable(old, focus_handles);
    }

    pub(super) fn remeasure_cell(&self, id: CellId) {
        if let Some(index) = self.index_of(id) {
            self.list_state.remeasure_items(index..index + 1);
        }
    }

    pub(super) fn reveal(&self, id: CellId) {
        if let Some(index) = self.index_of(id) {
            self.list_state.scroll_to_reveal_item(index);
        }
    }
}
