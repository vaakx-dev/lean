use gpui_kit::App;
use gpui_kit::component::Rope;
use serde_json::Value;

use super::Notebook;
use super::cell::Cell;
use super::document::CellKind;
use super::settings::SAVE_OUTPUTS;

pub(super) struct SavedCell {
    kind: CellKind,
    source: Rope,
    outputs: Vec<Value>,
    execution_count: Option<u64>,
}

impl Cell {
    fn saved_state(&self, cx: &App) -> SavedCell {
        SavedCell {
            kind: self.kind,
            source: self.editor.read(cx).text().clone(),
            outputs: self
                .outputs
                .iter()
                .map(|output| output.json.clone())
                .collect(),
            execution_count: self.execution_count,
        }
    }

    fn matches(&self, saved: &SavedCell, with_outputs: bool, cx: &App) -> bool {
        self.kind == saved.kind
            && *self.editor.read(cx).text() == saved.source
            && (!with_outputs || self.outputs_match(saved))
    }

    fn outputs_match(&self, saved: &SavedCell) -> bool {
        self.execution_count == saved.execution_count
            && self.outputs.len() == saved.outputs.len()
            && self
                .outputs
                .iter()
                .zip(&saved.outputs)
                .all(|(output, saved)| output.json == *saved)
    }
}

impl Notebook {
    pub(super) fn saved_cells(&self, cx: &App) -> Vec<SavedCell> {
        self.cells.iter().map(|cell| cell.saved_state(cx)).collect()
    }

    pub(super) fn has_unsaved_changes(&self, cx: &App) -> bool {
        let with_outputs = SAVE_OUTPUTS.get(cx);
        self.cells.len() != self.saved.len()
            || self
                .cells
                .iter()
                .zip(&self.saved)
                .any(|(cell, saved)| !cell.matches(saved, with_outputs, cx))
    }
}
