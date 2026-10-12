mod cell;
mod cell_options;
mod document;
mod editing;
mod execution;
mod kernel;
mod keymap;
mod loading;
mod output;
mod saved;
pub(crate) mod settings;
mod settings_sync;
mod status;
mod toolbar;
mod view;
mod viewer;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::*;
use serde_json::{Map, Value};

use super::{FileViewEvent, Plugin, Registry};
use crate::toolchain::{Toolchains, ToolchainsEvent};
use cell::{Cell, CellId};
use cell_options::{CellFont, cell_font};
use execution::Session;
use keymap::bind_keys;
use loading::LoadState;
use output::release_outputs;
use saved::SavedCell;
use status::KernelStatus;

const LIST_OVERDRAW: Pixels = px(600.);

pub(super) const PLUGIN: Plugin = Plugin {
    id: "notebook",
    name: "Jupyter Notebooks",
    description: "Open, edit and run .ipynb notebooks with a Python kernel.",
    activate,
};

fn activate(registry: &mut Registry, cx: &mut App) {
    bind_keys(cx);
    registry.register_viewer(viewer::NOTEBOOK_VIEWER);
}

pub(crate) struct Notebook {
    folder: PathBuf,
    language: String,
    fields: Arc<Map<String, Value>>,
    cells: Vec<Cell>,
    cell_font: CellFont,
    saved: Arc<[SavedCell]>,
    load_state: LoadState,
    selected: Option<CellId>,
    next_cell_id: usize,
    toolchains: Entity<Toolchains>,
    session: Option<Session>,
    awaiting_toolchains: bool,
    kernel_error: Option<SharedString>,
    queue: VecDeque<CellId>,
    focus_handle: FocusHandle,
    list_state: ListState,
    _loading: Task<()>,
    _toolchains_subscription: Subscription,
    _quit_subscription: Subscription,
    _release_subscription: Subscription,
    _settings: Vec<Subscription>,
}

impl EventEmitter<FileViewEvent> for Notebook {}

impl Notebook {
    fn new(
        folder: PathBuf,
        toolchains: Entity<Toolchains>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let toolchains_subscription = cx
            .subscribe(&toolchains, |notebook, _, event: &ToolchainsEvent, cx| {
                notebook.handle_toolchains_event(event, cx)
            });
        let quit_subscription = cx.on_app_quit(|notebook, _| notebook.kill_kernel());
        let release_subscription = cx.on_release(|notebook, cx| {
            let cells = std::mem::take(&mut notebook.cells);
            release_outputs(cells.into_iter().flat_map(|cell| cell.outputs), cx);
        });
        let setting_observers = Self::observe_settings(window, cx);
        Self {
            folder,
            language: String::new(),
            fields: Arc::default(),
            cells: Vec::new(),
            cell_font: cell_font(cx),
            saved: Arc::new([]),
            load_state: LoadState::Loading,
            selected: None,
            next_cell_id: 0,
            toolchains,
            session: None,
            awaiting_toolchains: false,
            kernel_error: None,
            queue: VecDeque::new(),
            focus_handle: cx.focus_handle(),
            list_state: ListState::new(0, ListAlignment::Top, LIST_OVERDRAW),
            _loading: Task::ready(()),
            _toolchains_subscription: toolchains_subscription,
            _quit_subscription: quit_subscription,
            _release_subscription: release_subscription,
            _settings: setting_observers,
        }
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
