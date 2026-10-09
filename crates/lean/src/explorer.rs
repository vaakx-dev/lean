mod context_menu;
mod file_operations;
mod keyboard;
mod loading;
mod panel;
mod rows;
mod sync;
mod tree;
mod view;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui_kit::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString, Subscription,
    UniformListScrollHandle, Window,
};

use crate::editor::EditorAreaEvent;
use crate::plugins::{FileDecorations, PanelEvent, WorkspaceContext};
use crate::project::{DirectoryEntry, FilesChanged, Project};
use file_operations::InlineEdit;
use loading::FolderLoads;

pub(crate) use keyboard::bind_keys;
pub(crate) use panel::PANEL;

struct VisibleEntry {
    path: PathBuf,
    name: SharedString,
    depth: usize,
    is_dir: bool,
    is_draft: bool,
}

pub(crate) struct Explorer {
    project: Entity<Project>,
    decorations: Entity<FileDecorations>,
    root: PathBuf,
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    root_expanded: bool,
    expanded: HashSet<PathBuf>,
    children: HashMap<PathBuf, Vec<DirectoryEntry>>,
    visible: Vec<VisibleEntry>,
    selected: Option<PathBuf>,
    editing: Option<InlineEdit>,
    loads: FolderLoads,
    _subscriptions: [Subscription; 3],
}

impl EventEmitter<PanelEvent> for Explorer {}

impl Explorer {
    fn new(workspace: &WorkspaceContext, window: &Window, cx: &mut Context<Self>) -> Self {
        let project = workspace.project.clone();
        let subscriptions = [
            cx.observe(&workspace.decorations, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &project,
                window,
                |explorer, _, event: &FilesChanged, window, cx| {
                    explorer.apply_fs_changes(&event.0, window, cx)
                },
            ),
            cx.subscribe(
                &workspace.editor_area,
                |explorer, _, event: &EditorAreaEvent, cx| {
                    if let EditorAreaEvent::FileRecreated(path) = event {
                        explorer.show_recreated_file(path, cx);
                    }
                },
            ),
        ];
        let root = project.read(cx).root().to_path_buf();
        let mut explorer = Self {
            project,
            decorations: workspace.decorations.clone(),
            root: root.clone(),
            focus_handle: cx.focus_handle(),
            scroll_handle: UniformListScrollHandle::new(),
            root_expanded: true,
            expanded: HashSet::new(),
            children: HashMap::new(),
            visible: Vec::new(),
            selected: None,
            editing: None,
            loads: FolderLoads::default(),
            _subscriptions: subscriptions,
        };
        explorer.load_directory(root, cx);
        explorer
    }
}

impl Focusable for Explorer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn chevron_icon(is_expanded: bool) -> &'static str {
    if is_expanded {
        "icons/chevron-down.svg"
    } else {
        "icons/chevron-right.svg"
    }
}
