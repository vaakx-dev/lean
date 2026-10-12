mod compact;
mod context_menu;
mod file_operations;
mod keyboard;
mod loading;
mod options;
mod panel;
mod revealing;
mod rows;
pub(crate) mod settings;
mod sorting;
mod subscriptions;
mod sync;
mod tree;
mod view;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use gpui_kit::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString, Subscription,
    UniformListScrollHandle, Window,
};

use crate::editor::EditorArea;
use crate::plugins::{FileDecorations, PanelEvent, WorkspaceContext};
use crate::project::{DirectoryEntry, Project};
use file_operations::InlineEdit;
use loading::FolderLoads;
use options::ExplorerOptions;

pub(crate) use keyboard::bind_keys;
pub(crate) use panel::PANEL;
pub(crate) use revealing::reveal_active_file;

struct VisibleEntry {
    path: Arc<Path>,
    name: SharedString,
    depth: usize,
    is_dir: bool,
    is_draft: bool,
    is_ignored: bool,
}

pub(crate) struct Explorer {
    project: Entity<Project>,
    editor_area: Entity<EditorArea>,
    decorations: Entity<FileDecorations>,
    root: PathBuf,
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    options: ExplorerOptions,
    root_expanded: bool,
    expanded: HashSet<PathBuf>,
    unfolding: HashSet<PathBuf>,
    children: HashMap<PathBuf, Vec<DirectoryEntry>>,
    visible: Vec<VisibleEntry>,
    selected: Option<PathBuf>,
    pending_reveal: Option<PathBuf>,
    followed: Option<PathBuf>,
    editing: Option<InlineEdit>,
    loads: FolderLoads,
    opened_at: Instant,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for Explorer {}

impl Explorer {
    fn new(workspace: &WorkspaceContext, window: &Window, cx: &mut Context<Self>) -> Self {
        let project = workspace.project.clone();
        let root = project.read(cx).root().to_path_buf();
        let mut explorer = Self {
            project,
            editor_area: workspace.editor_area.clone(),
            decorations: workspace.decorations.clone(),
            root: root.clone(),
            focus_handle: cx.focus_handle(),
            scroll_handle: UniformListScrollHandle::new(),
            options: ExplorerOptions::read(cx),
            root_expanded: true,
            expanded: HashSet::new(),
            unfolding: HashSet::new(),
            children: HashMap::new(),
            visible: Vec::new(),
            selected: None,
            pending_reveal: None,
            followed: None,
            editing: None,
            loads: FolderLoads::default(),
            opened_at: Instant::now(),
            _subscriptions: subscriptions::subscribe(workspace, window, cx),
        };
        explorer.load_directory(root, window, cx);
        explorer.follow_active_editor(window, cx);
        explorer
    }
}

impl Focusable for Explorer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
