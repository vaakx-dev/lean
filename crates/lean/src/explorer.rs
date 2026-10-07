mod keyboard;
mod rows;
mod tree;
mod view;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui_kit::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString, Subscription,
    UniformListScrollHandle,
};

use crate::project::{DirectoryEntry, Project, ProjectEvent};

pub(crate) use keyboard::bind_keys;

pub(crate) enum ExplorerEvent {
    OpenFile(PathBuf),
}

struct VisibleEntry {
    path: PathBuf,
    name: SharedString,
    depth: usize,
    is_dir: bool,
}

pub(crate) struct Explorer {
    project: Entity<Project>,
    root: PathBuf,
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    root_expanded: bool,
    expanded: HashSet<PathBuf>,
    children: HashMap<PathBuf, Vec<DirectoryEntry>>,
    visible: Vec<VisibleEntry>,
    selected: Option<PathBuf>,
    _project_subscription: Subscription,
}

impl EventEmitter<ExplorerEvent> for Explorer {}

impl Explorer {
    pub(crate) fn new(project: Entity<Project>, cx: &mut Context<Self>) -> Self {
        let project_subscription =
            cx.subscribe(&project, |_, _, event: &ProjectEvent, cx| match event {
                ProjectEvent::GitChanged => cx.notify(),
            });
        let root = project.read(cx).root().to_path_buf();
        let mut explorer = Self {
            project,
            root: root.clone(),
            focus_handle: cx.focus_handle(),
            scroll_handle: UniformListScrollHandle::new(),
            root_expanded: true,
            expanded: HashSet::new(),
            children: HashMap::new(),
            visible: Vec::new(),
            selected: None,
            _project_subscription: project_subscription,
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
