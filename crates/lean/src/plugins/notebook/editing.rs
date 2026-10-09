use gpui_kit::*;

use super::Notebook;
use super::cell::CellId;
use super::document::{CellData, CellKind};
use crate::plugins::FileViewEvent;

impl Notebook {
    pub(super) fn select(&mut self, id: CellId, cx: &mut Context<Self>) {
        if self.selected == Some(id) {
            return;
        }
        self.selected = Some(id);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    pub(super) fn select_offset(&mut self, offset: isize, cx: &mut Context<Self>) {
        let Some(last) = self.cells.len().checked_sub(1) else {
            return;
        };
        let index = match self.selected.and_then(|id| self.index_of(id)) {
            Some(index) => index.saturating_add_signed(offset).min(last),
            None => 0,
        };
        let id = self.cells[index].id;
        self.select(id, cx);
        self.reveal(id);
    }

    pub(super) fn selected_index(&self) -> Option<usize> {
        self.selected.and_then(|id| self.index_of(id))
    }

    pub(super) fn insert_cell(
        &mut self,
        index: usize,
        kind: CellKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let cell = self.create_cell(CellData::new(kind), window, cx);
        let id = cell.id;
        let index = index.min(self.cells.len());
        self.cells.insert(index, cell);
        self.splice_rows(index..index, index..index + 1, cx);
        cx.emit(FileViewEvent::Edited);
        self.edit(id, window, cx);
    }

    pub(super) fn insert_below_selected(
        &mut self,
        kind: CellKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self
            .selected_index()
            .map_or(self.cells.len(), |index| index + 1);
        self.insert_cell(index, kind, window, cx);
    }

    pub(super) fn delete_cell(&mut self, id: CellId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        self.cells.remove(index);
        self.splice_rows(index..index + 1, 0..0, cx);
        self.queue.retain(|queued| *queued != id);
        self.selected = self
            .cells
            .get(index)
            .or_else(|| self.cells.last())
            .map(|cell| cell.id);
        window.focus(&self.focus_handle, cx);
        cx.emit(FileViewEvent::Edited);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    pub(super) fn move_cell(&mut self, id: CellId, offset: isize, cx: &mut Context<Self>) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let Some(target) = index
            .checked_add_signed(offset)
            .filter(|target| *target < self.cells.len())
        else {
            return;
        };
        let cell = self.cells.remove(index);
        self.cells.insert(target, cell);
        let rows = index.min(target)..index.max(target) + 1;
        self.splice_rows(rows.clone(), rows, cx);
        self.list_state.scroll_to_reveal_item(target);
        cx.emit(FileViewEvent::Edited);
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    pub(super) fn clear_cell_outputs(&mut self, id: CellId, cx: &mut Context<Self>) {
        if let Some(cell) = self.cell_mut(id)
            && !cell.is_pending()
        {
            cell.reset();
            self.remeasure_cell(id);
            cx.emit(FileViewEvent::Edited);
            cx.notify();
        }
    }

    pub(super) fn clear_all_outputs(&mut self, cx: &mut Context<Self>) {
        for cell in self.cells.iter_mut().filter(|cell| !cell.is_pending()) {
            cell.reset();
        }
        self.list_state.remeasure();
        cx.emit(FileViewEvent::Edited);
        cx.notify();
    }

    pub(super) fn edit(&mut self, id: CellId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(cell) = self.cell_mut(id) else {
            return;
        };
        cell.editing = true;
        let editor = cell.editor.clone();
        editor.update(cx, |editor, cx| editor.focus(window, cx));
        self.select(id, cx);
        self.reveal(id);
        cx.notify();
    }

    pub(super) fn stop_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for (index, cell) in self.cells.iter_mut().enumerate() {
            if cell.kind == CellKind::Markdown && cell.editing {
                cell.editing = false;
                self.list_state.remeasure_items(index..index + 1);
            }
        }
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    pub(super) fn advance(&mut self, id: CellId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        match self.cells.get(index + 1) {
            Some(next) if next.kind == CellKind::Code => {
                let next = next.id;
                self.edit(next, window, cx);
            }
            Some(next) => {
                let next = next.id;
                self.stop_editing(window, cx);
                self.select(next, cx);
                self.reveal(next);
            }
            None => self.insert_cell(index + 1, CellKind::Code, window, cx),
        }
    }
}
