use std::path::{Path, PathBuf};

use gpui_kit::{App, Context, ScrollStrategy, Window};

use super::Explorer;
use crate::plugins::WorkspaceContext;
use crate::project::ProjectEvent;

pub(crate) fn reveal_active_file(workspace: &WorkspaceContext, cx: &mut App) -> bool {
    let active = workspace
        .editor_area
        .read(cx)
        .active_path()
        .map(Path::to_path_buf);
    let Some(path) = active else {
        return false;
    };
    workspace
        .project
        .update(cx, |_, cx| cx.emit(ProjectEvent::RevealRequested(path)));
    true
}

impl Explorer {
    pub(super) fn follow_active_editor(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.options.auto_reveal {
            return;
        }
        let active = self
            .editor_area
            .read(cx)
            .active_path()
            .map(Path::to_path_buf);
        if active == self.followed {
            return;
        }
        self.followed = active.clone();
        if let Some(path) = active {
            self.reveal_path(path, window, cx);
        }
    }

    pub(super) fn reveal_path(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        if path == self.root || !path.starts_with(&self.root) {
            return;
        }
        self.root_expanded = true;
        let folders: Vec<PathBuf> = path
            .ancestors()
            .skip(1)
            .take_while(|folder| folder.starts_with(&self.root))
            .map(Path::to_path_buf)
            .collect();
        let mut unloaded = Vec::new();
        for folder in folders {
            if folder != self.root {
                self.expanded.insert(folder.clone());
            }
            if !self.children.contains_key(&folder) {
                unloaded.push(folder);
            }
        }
        self.load_directories(unloaded, window, cx);
        self.selected = Some(path.clone());
        self.pending_reveal = Some(path);
        self.rebuild_visible();
        self.scroll_to_pending_reveal();
        cx.notify();
    }

    pub(super) fn scroll_to_pending_reveal(&mut self) {
        let Some(path) = self.pending_reveal.as_ref() else {
            return;
        };
        let index = self
            .visible
            .iter()
            .position(|entry| !entry.is_draft && *entry.path == **path);
        if let Some(index) = index {
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
            self.pending_reveal = None;
        } else if self.loads.is_idle() {
            self.pending_reveal = None;
        }
    }
}
