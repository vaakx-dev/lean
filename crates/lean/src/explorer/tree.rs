use std::path::{Path, PathBuf};

use gpui_kit::{Context, ScrollStrategy, SharedString, Window};

use super::{Explorer, VisibleEntry};
use crate::app::actions::{CollapseAllFolders, RefreshExplorer};
use crate::project::DirectoryEntry;

impl Explorer {
    pub(super) fn toggle_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.expanded.remove(&path) {
            self.rebuild_visible();
            cx.notify();
        } else {
            self.expand_folder(path, cx);
        }
    }

    pub(super) fn expand_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.expanded.insert(path.clone());
        if self.children.contains_key(&path) {
            self.rebuild_visible();
            cx.notify();
            if self.project.read(cx).watches_folder(&path) {
                return;
            }
        }
        self.load_directory(path, cx);
    }

    pub(super) fn toggle_root(&mut self, cx: &mut Context<Self>) {
        self.root_expanded = !self.root_expanded;
        self.rebuild_visible();
        cx.notify();
    }

    pub(super) fn refresh(&mut self, _: &RefreshExplorer, _: &mut Window, cx: &mut Context<Self>) {
        self.project.update(cx, |project, _| project.rescan());
    }

    pub(super) fn show_recreated_file(&mut self, path: &Path, cx: &mut Context<Self>) {
        let loaded_folder = path
            .ancestors()
            .skip(1)
            .find(|folder| self.children.contains_key(*folder))
            .map(Path::to_path_buf);
        if let Some(folder) = loaded_folder {
            self.load_directory(folder, cx);
        }
    }

    pub(super) fn collapse_all(
        &mut self,
        _: &CollapseAllFolders,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.expanded.clear();
        self.rebuild_visible();
        cx.notify();
    }

    pub(super) fn reveal(&mut self, path: PathBuf) {
        let index = self
            .visible
            .iter()
            .position(|entry| !entry.is_draft && entry.path == path);
        if let Some(index) = index {
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
        }
        self.selected = Some(path);
    }

    pub(super) fn rebuild_visible(&mut self) {
        let mut visible = Vec::new();
        if self.root_expanded {
            self.push_children(&self.root, 0, &mut visible);
        }
        self.visible = visible;
    }

    fn push_children(&self, path: &Path, depth: usize, visible: &mut Vec<VisibleEntry>) {
        let children = self
            .children
            .get(path)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let draft = self
            .draft_in(path)
            .map(|is_dir| (draft_position(children, is_dir), is_dir));
        for (index, child) in children.iter().enumerate() {
            if let Some((position, is_dir)) = draft
                && position == index
            {
                visible.push(draft_entry(path, depth, is_dir));
            }
            visible.push(VisibleEntry {
                path: child.path.clone(),
                name: child.name.clone(),
                depth,
                is_dir: child.is_dir,
                is_draft: false,
            });
            if child.is_dir && self.expanded.contains(&child.path) {
                self.push_children(&child.path, depth + 1, visible);
            }
        }
        if let Some((position, is_dir)) = draft
            && position == children.len()
        {
            visible.push(draft_entry(path, depth, is_dir));
        }
    }
}

fn draft_position(children: &[DirectoryEntry], is_dir: bool) -> usize {
    if is_dir {
        0
    } else {
        children.iter().take_while(|child| child.is_dir).count()
    }
}

fn draft_entry(folder: &Path, depth: usize, is_dir: bool) -> VisibleEntry {
    VisibleEntry {
        path: folder.to_path_buf(),
        name: SharedString::default(),
        depth,
        is_dir,
        is_draft: true,
    }
}
