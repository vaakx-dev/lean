use std::path::{Path, PathBuf};

use gpui_kit::{Context, SharedString, Window};

use super::{Explorer, VisibleEntry};
use crate::app::actions::{CollapseAllFolders, RefreshExplorer};
use crate::project::DirectoryEntry;

impl Explorer {
    pub(super) fn toggle_folder(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        if self.expanded.remove(&path) {
            self.rebuild_visible();
            cx.notify();
        } else {
            self.expand_folder(path, window, cx);
        }
    }

    pub(super) fn expand_folder(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        self.expanded.insert(path.clone());
        if self.children.contains_key(&path) {
            self.rebuild_visible();
            cx.notify();
            self.unfold_single_child(&path, window, cx);
            if !self.crosses_symlink(&path) && self.project.read(cx).watches_folder(&path) {
                return;
            }
        } else if self.options.compact_folders {
            self.unfolding.insert(path.clone());
        }
        self.load_directory(path, window, cx);
    }

    pub(super) fn toggle_root(&mut self, cx: &mut Context<Self>) {
        self.root_expanded = !self.root_expanded;
        self.rebuild_visible();
        cx.notify();
    }

    pub(super) fn refresh(&mut self, _: &RefreshExplorer, _: &mut Window, cx: &mut Context<Self>) {
        self.project.update(cx, |project, _| project.rescan());
    }

    pub(super) fn show_recreated_file(
        &mut self,
        path: &Path,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let loaded_folder = path
            .ancestors()
            .skip(1)
            .find(|folder| self.children.contains_key(*folder))
            .map(Path::to_path_buf);
        if let Some(folder) = loaded_folder {
            self.load_directory(folder, window, cx);
        }
    }

    pub(super) fn collapse_all(
        &mut self,
        _: &CollapseAllFolders,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.expanded.clear();
        self.unfolding.clear();
        self.rebuild_visible();
        cx.notify();
    }

    fn crosses_symlink(&self, folder: &Path) -> bool {
        folder
            .ancestors()
            .take_while(|ancestor| *ancestor != self.root.as_path())
            .any(|ancestor| {
                ancestor
                    .parent()
                    .and_then(|parent| self.children.get(parent))
                    .and_then(|children| children.iter().find(|child| *child.path == *ancestor))
                    .is_none_or(|child| child.is_symlink)
            })
    }

    pub(super) fn rebuild_visible(&mut self) {
        let mut visible = Vec::new();
        if self.root_expanded {
            self.push_children(&self.root, 0, &mut visible);
        }
        self.visible = visible;
    }

    pub(super) fn shown_children<'a>(
        &'a self,
        folder: &Path,
    ) -> impl Iterator<Item = &'a DirectoryEntry> + use<'a> {
        let options = &self.options;
        self.children
            .get(folder)
            .into_iter()
            .flatten()
            .filter(move |entry| options.shows(entry))
    }

    fn push_children(&self, folder: &Path, depth: usize, visible: &mut Vec<VisibleEntry>) {
        let children: Vec<&DirectoryEntry> = self.shown_children(folder).collect();
        let draft = self
            .draft_in(folder)
            .map(|is_dir| (draft_position(&children, is_dir), is_dir));
        for (index, child) in children.iter().enumerate() {
            if let Some((position, is_dir)) = draft
                && position == index
            {
                visible.push(draft_entry(folder, depth, is_dir));
            }
            let row = self.entry_row(child, depth);
            let opened =
                (row.is_dir && self.expanded.contains(&*row.path)).then(|| row.path.clone());
            visible.push(row);
            if let Some(opened) = opened {
                self.push_children(&opened, depth + 1, visible);
            }
        }
        if let Some((position, is_dir)) = draft
            && position == children.len()
        {
            visible.push(draft_entry(folder, depth, is_dir));
        }
    }
}

fn draft_position(children: &[&DirectoryEntry], is_dir: bool) -> usize {
    if is_dir {
        0
    } else {
        children.iter().take_while(|child| child.is_dir).count()
    }
}

fn draft_entry(folder: &Path, depth: usize, is_dir: bool) -> VisibleEntry {
    VisibleEntry {
        path: folder.into(),
        name: SharedString::default(),
        depth,
        is_dir,
        is_draft: true,
        is_ignored: false,
    }
}
