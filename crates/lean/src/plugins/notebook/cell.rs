mod view;

use std::time::{Duration, Instant};

use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::*;
use serde_json::{Map, Value};

use super::Notebook;
use super::cell_options::cell_tab_size;
use super::document::{CellData, CellKind};
use super::output::Output;
use super::settings::{LINE_NUMBERS, SOFT_WRAP};
use crate::plugins::FileViewEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct CellId(pub(super) usize);

pub(super) enum RunState {
    Idle,
    Queued,
    Running,
    Done { elapsed: Duration, success: bool },
}

pub(super) struct Cell {
    pub(super) id: CellId,
    pub(super) kind: CellKind,
    pub(super) editor: Entity<EditorState>,
    pub(super) outputs: Vec<Output>,
    pub(super) execution_count: Option<u64>,
    pub(super) run_state: RunState,
    pub(super) editing: bool,
    pub(super) markdown: SharedString,
    pub(super) verbatim: Option<Value>,
    clear_pending: bool,
    fields: Map<String, Value>,
    _subscription: Subscription,
}

impl Cell {
    pub(super) fn is_pending(&self) -> bool {
        matches!(self.run_state, RunState::Queued | RunState::Running)
    }

    pub(super) fn has_error(&self) -> bool {
        self.outputs.iter().any(Output::is_error)
    }

    pub(super) fn to_data(&self, with_outputs: bool, cx: &App) -> CellData {
        let outputs = if with_outputs {
            self.outputs
                .iter()
                .map(|output| output.json.clone())
                .collect()
        } else {
            Vec::new()
        };
        CellData {
            kind: self.kind,
            source: self.editor.read(cx).value().to_string(),
            outputs,
            execution_count: self.execution_count.filter(|_| with_outputs),
            fields: self.fields.clone(),
            verbatim: self.verbatim.clone(),
        }
    }

    pub(super) fn rebuild_outputs(&mut self, cx: &mut Context<Notebook>) {
        self.outputs = self
            .outputs
            .iter()
            .map(|output| Output::new(output.json.clone(), cx))
            .collect();
    }

    pub(super) fn start_run(&mut self) {
        self.outputs.clear();
        self.clear_pending = false;
        self.execution_count = None;
        self.run_state = RunState::Running;
    }

    pub(super) fn finish_run(&mut self, started: Instant) {
        self.run_state = RunState::Done {
            elapsed: started.elapsed(),
            success: !self.has_error(),
        };
    }

    pub(super) fn push_output(&mut self, json: Value, cx: &mut Context<Notebook>) {
        if self.clear_pending {
            self.outputs.clear();
            self.clear_pending = false;
        }
        if let Some(last) = self.outputs.last_mut()
            && last.merge_stream(&json)
        {
            return;
        }
        self.outputs.push(Output::new(json, cx));
    }

    pub(super) fn clear_outputs(&mut self, wait: bool) {
        if wait {
            self.clear_pending = true;
        } else {
            self.outputs.clear();
        }
    }

    fn sync_markdown(&mut self, cx: &App) {
        self.markdown = if self.kind == CellKind::Markdown {
            self.editor.read(cx).value()
        } else {
            SharedString::default()
        };
    }

    pub(super) fn reset(&mut self) {
        self.outputs.clear();
        self.clear_pending = false;
        self.execution_count = None;
        self.run_state = RunState::Idle;
    }
}

impl Notebook {
    pub(super) fn create_cell(
        &mut self,
        data: CellData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Cell {
        let id = CellId(self.next_cell_id);
        self.next_cell_id += 1;
        let language = editor_language(data.kind, &self.language);
        let editing = data.kind == CellKind::Markdown && data.source.is_empty();
        let line_numbers = LINE_NUMBERS.get(cx);
        let soft_wrap = SOFT_WRAP.get(cx);
        let tab_size = cell_tab_size(cx);
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language(language)
                .line_number(line_numbers)
                .folding(false)
                .indent_guides(false)
                .soft_wrap(soft_wrap)
                .searchable(false)
                .scroll_beyond_last_line(Some(0))
                .tab_size(tab_size)
                .default_value(data.source)
        });
        let subscription = cx.subscribe(&editor, move |notebook, _, event: &InputEvent, cx| {
            notebook.handle_cell_input(id, event, cx)
        });
        let mut cell = Cell {
            id,
            kind: data.kind,
            editor,
            outputs: data
                .outputs
                .into_iter()
                .map(|json| Output::new(json, cx))
                .collect(),
            execution_count: data.execution_count,
            run_state: RunState::Idle,
            editing,
            markdown: SharedString::default(),
            verbatim: data.verbatim,
            clear_pending: false,
            fields: data.fields,
            _subscription: subscription,
        };
        cell.sync_markdown(cx);
        cell
    }

    pub(super) fn set_cell_kind(&mut self, id: CellId, kind: CellKind, cx: &mut Context<Self>) {
        let language = editor_language(kind, &self.language);
        let Some(cell) = self.cell_mut(id) else {
            return;
        };
        if cell.kind == kind || cell.is_pending() || cell.verbatim.is_some() {
            return;
        }
        cell.kind = kind;
        cell.editing = false;
        cell.reset();
        cell.sync_markdown(cx);
        cell.editor
            .update(cx, |editor, cx| editor.set_highlighter(language, cx));
        self.remeasure_cell(id);
        cx.emit(FileViewEvent::Edited);
        cx.notify();
    }

    fn handle_cell_input(&mut self, id: CellId, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Change => {
                if let Some(cell) = self.cell_mut(id) {
                    cell.sync_markdown(cx);
                }
                cx.emit(FileViewEvent::Edited);
                cx.notify();
            }
            InputEvent::Focus => self.select(id, cx),
            InputEvent::PressEnter { .. } | InputEvent::Blur => {}
        }
    }
}

fn editor_language(kind: CellKind, language: &str) -> SharedString {
    match kind {
        CellKind::Code => SharedString::from(language.to_string()),
        CellKind::Markdown => "markdown".into(),
        CellKind::Raw => "text".into(),
    }
}
