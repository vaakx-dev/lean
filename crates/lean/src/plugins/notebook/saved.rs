use std::sync::Arc;

use gpui_kit::App;
use gpui_kit::component::Rope;
use serde_json::{Map, Value};

use super::Notebook;
use super::cell::{Cell, CellId};
use super::document::{CellData, CellKind, Document};
use super::settings::SAVE_OUTPUTS;

pub(super) struct SavedCell {
    id: CellId,
    kind: CellKind,
    source: Rope,
    outputs: Vec<Arc<Value>>,
    execution_count: Option<u64>,
    fields: Arc<Map<String, Value>>,
    verbatim: Option<Arc<Value>>,
}

impl SavedCell {
    fn to_data(&self, with_outputs: bool) -> CellData {
        let outputs = if with_outputs {
            self.outputs.iter().map(|json| Value::clone(json)).collect()
        } else {
            Vec::new()
        };
        CellData {
            kind: self.kind,
            source: self.source.to_string(),
            outputs,
            execution_count: self.execution_count.filter(|_| with_outputs),
            fields: Map::clone(&self.fields),
            verbatim: self.verbatim.as_deref().cloned(),
        }
    }
}

pub(super) fn serialize(
    fields: &Map<String, Value>,
    cells: &[SavedCell],
    with_outputs: bool,
) -> String {
    Document {
        cells: cells
            .iter()
            .map(|cell| cell.to_data(with_outputs))
            .collect(),
        fields: fields.clone(),
    }
    .serialize()
}

impl Cell {
    fn saved_state(&self, cx: &App) -> SavedCell {
        SavedCell {
            id: self.id,
            kind: self.kind,
            source: self.editor.read(cx).text().clone(),
            outputs: self
                .outputs
                .iter()
                .map(|output| output.json.clone())
                .collect(),
            execution_count: self.execution_count,
            fields: self.fields.clone(),
            verbatim: self.verbatim.clone(),
        }
    }

    fn source_matches(&self, saved: Option<&SavedCell>, cx: &App) -> bool {
        saved.is_some_and(|saved| *self.editor.read(cx).text() == saved.source)
    }

    fn matches(&self, saved: &SavedCell, with_outputs: bool) -> bool {
        self.id == saved.id
            && self.kind == saved.kind
            && self.source_saved
            && (!with_outputs || self.outputs_match(saved))
    }

    fn outputs_match(&self, saved: &SavedCell) -> bool {
        self.execution_count == saved.execution_count
            && self.outputs.len() == saved.outputs.len()
            && self
                .outputs
                .iter()
                .zip(&saved.outputs)
                .all(|(output, saved)| Arc::ptr_eq(&output.json, saved))
    }
}

impl Notebook {
    pub(super) fn saved_cells(&self, cx: &App) -> Arc<[SavedCell]> {
        self.cells.iter().map(|cell| cell.saved_state(cx)).collect()
    }

    pub(super) fn mark_saved(&mut self, saved: Arc<[SavedCell]>, cx: &App) {
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.source_saved = cell.source_matches(saved_entry(&saved, index, cell.id), cx);
        }
        self.saved = saved;
    }

    pub(super) fn refresh_source_saved(&mut self, id: CellId, cx: &App) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let cell = &self.cells[index];
        let source_saved = cell.source_matches(saved_entry(&self.saved, index, id), cx);
        self.cells[index].source_saved = source_saved;
    }

    pub(super) fn has_unsaved_changes(&self, cx: &App) -> bool {
        let with_outputs = SAVE_OUTPUTS.get(cx);
        self.cells.len() != self.saved.len()
            || self
                .cells
                .iter()
                .zip(self.saved.iter())
                .any(|(cell, saved)| !cell.matches(saved, with_outputs))
    }
}

fn saved_entry(saved: &[SavedCell], index: usize, id: CellId) -> Option<&SavedCell> {
    saved
        .get(index)
        .filter(|entry| entry.id == id)
        .or_else(|| saved.iter().find(|entry| entry.id == id))
}
