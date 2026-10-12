mod view;

use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::*;
use serde_json::{Map, Value};

use super::Notebook;
use super::cell_options::cell_tab_size;
use super::document::{CellData, CellKind};
use super::output::{Output, OutputDecoder, release_outputs};
use super::settings::{LINE_NUMBERS, SOFT_WRAP};
use crate::plugins::FileViewEvent;
use view::RowCache;

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
    pub(super) verbatim: Option<Arc<Value>>,
    pub(super) fields: Arc<Map<String, Value>>,
    pub(super) source_saved: bool,
    pub(super) revision: u64,
    pub(super) rows: RowCache,
    clear_pending: bool,
    _subscription: Subscription,
}

impl Cell {
    pub(super) fn is_pending(&self) -> bool {
        matches!(self.run_state, RunState::Queued | RunState::Running)
    }

    pub(super) fn has_error(&self) -> bool {
        self.outputs.iter().any(Output::is_error)
    }

    pub(super) fn rebuild_outputs(&mut self, decoder: &OutputDecoder) {
        for output in &mut self.outputs {
            output.relimit(decoder);
        }
    }

    pub(super) fn start_run(&mut self, cx: &mut App) {
        self.discard_outputs(cx);
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

    pub(super) fn push_output(&mut self, output: Output, cx: &mut App) {
        if self.clear_pending {
            self.discard_outputs(cx);
            self.clear_pending = false;
        }
        if let Some(last) = self.outputs.last_mut()
            && last.merge_stream(&output)
        {
            return;
        }
        self.outputs.push(output);
    }

    pub(super) fn clear_outputs(&mut self, wait: bool, cx: &mut App) {
        if wait {
            self.clear_pending = true;
        } else {
            self.discard_outputs(cx);
        }
    }

    pub(super) fn finish_editing(&mut self, cx: &App) {
        self.editing = false;
        self.sync_markdown(cx);
    }

    pub(super) fn reset(&mut self, cx: &mut App) {
        self.discard_outputs(cx);
        self.clear_pending = false;
        self.execution_count = None;
        self.run_state = RunState::Idle;
    }

    pub(super) fn discard_outputs(&mut self, cx: &mut App) {
        release_outputs(mem::take(&mut self.outputs), cx);
    }

    fn sync_markdown(&mut self, cx: &App) {
        self.markdown = if self.kind == CellKind::Markdown {
            self.editor.read(cx).value()
        } else {
            SharedString::default()
        };
    }
}

impl Notebook {
    pub(super) fn create_cell(
        &mut self,
        data: CellData,
        outputs: Vec<Output>,
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
            outputs,
            execution_count: data.execution_count,
            run_state: RunState::Idle,
            editing,
            markdown: SharedString::default(),
            verbatim: data.verbatim.map(Arc::new),
            fields: Arc::new(data.fields),
            source_saved: false,
            revision: 0,
            rows: RowCache::default(),
            clear_pending: false,
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
        cell.reset(cx);
        cell.finish_editing(cx);
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
                    cell.revision += 1;
                }
                self.refresh_source_saved(id, cx);
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
