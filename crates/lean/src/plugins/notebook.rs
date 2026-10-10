mod cell;
mod cell_options;
mod document;
mod editing;
mod execution;
mod kernel;
mod keymap;
mod output;
mod rows;
mod saved;
pub(crate) mod settings;
mod settings_sync;
mod status;
mod toolbar;
mod view;
mod viewer;

use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::*;
use serde_json::{Map, Value};

use super::{FileViewEvent, Plugin, PluginContext};
use crate::editor::file_text::LineEnding;
use crate::toolchain::{Toolchains, ToolchainsEvent};
use cell::{Cell, CellId};
use cell_options::{CellFont, cell_font};
use document::Document;
use execution::Session;
use keymap::bind_keys;
use saved::SavedCell;
use settings::SAVE_OUTPUTS;
use status::KernelStatus;

const LIST_OVERDRAW: Pixels = px(600.);

pub(super) const PLUGIN: Plugin = Plugin {
    id: "notebook",
    name: "Jupyter Notebooks",
    description: "Open, edit and run .ipynb notebooks with a Python kernel.",
    activate,
};

fn activate(context: &mut PluginContext, cx: &mut App) {
    bind_keys(cx);
    context.register_viewer(viewer::NOTEBOOK_VIEWER);
}

pub(crate) struct Notebook {
    folder: PathBuf,
    line_ending: LineEnding,
    language: String,
    fields: Map<String, Value>,
    cells: Vec<Cell>,
    cell_font: CellFont,
    saved: Vec<SavedCell>,
    selected: Option<CellId>,
    next_cell_id: usize,
    toolchains: Entity<Toolchains>,
    session: Option<Session>,
    awaiting_toolchains: bool,
    kernel_error: Option<SharedString>,
    queue: VecDeque<CellId>,
    focus_handle: FocusHandle,
    list_state: ListState,
    _toolchains_subscription: Subscription,
    _quit_subscription: Subscription,
    _settings: Vec<Subscription>,
}

impl EventEmitter<FileViewEvent> for Notebook {}

impl Notebook {
    fn new(
        folder: PathBuf,
        document: Document,
        line_ending: LineEnding,
        toolchains: Entity<Toolchains>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let language = document.language();
        let toolchains_subscription = cx
            .subscribe(&toolchains, |notebook, _, event: &ToolchainsEvent, cx| {
                notebook.handle_toolchains_event(event, cx)
            });
        let quit_subscription = cx.on_app_quit(|notebook, _| notebook.kill_kernel());
        let setting_observers = Self::observe_settings(window, cx);
        let mut notebook = Self {
            folder,
            line_ending,
            language,
            fields: document.fields,
            cells: Vec::new(),
            cell_font: cell_font(cx),
            saved: Vec::new(),
            selected: None,
            next_cell_id: 0,
            toolchains,
            session: None,
            awaiting_toolchains: false,
            kernel_error: None,
            queue: VecDeque::new(),
            focus_handle: cx.focus_handle(),
            list_state: ListState::new(0, ListAlignment::Top, LIST_OVERDRAW),
            _toolchains_subscription: toolchains_subscription,
            _quit_subscription: quit_subscription,
            _settings: setting_observers,
        };
        let cells = document
            .cells
            .into_iter()
            .map(|data| notebook.create_cell(data, window, cx))
            .collect();
        notebook.cells = cells;
        notebook.saved = notebook.saved_cells(cx);
        notebook.splice_rows(0..0, 0..notebook.cells.len() + 1, cx);
        notebook.selected = notebook.cells.first().map(|cell| cell.id);
        notebook
    }

    fn content(&self, cx: &App) -> Result<String> {
        let with_outputs = SAVE_OUTPUTS.get(cx);
        Document {
            cells: self
                .cells
                .iter()
                .map(|cell| cell.to_data(with_outputs, cx))
                .collect(),
            fields: self.fields.clone(),
        }
        .serialize()
    }

    fn cell(&self, id: CellId) -> Option<&Cell> {
        self.cells.iter().find(|cell| cell.id == id)
    }

    fn cell_mut(&mut self, id: CellId) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|cell| cell.id == id)
    }

    fn index_of(&self, id: CellId) -> Option<usize> {
        self.cells.iter().position(|cell| cell.id == id)
    }
}

impl Focusable for Notebook {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
