use std::path::{Path, PathBuf};

use gpui_kit::{Context, Window};

use super::{Explorer, VisibleEntry};
use crate::app::actions::{CollapseAllFolders, RefreshExplorer};
use crate::project::read_directory;

impl Explorer {
    pub(super) fn load_directory(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let task = read_directory(&path, cx);
        cx.spawn(async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update(cx, |explorer, cx| {
                match result {
                    Ok(entries) => {
                        explorer.children.insert(path, entries);
                    }
                    Err(error) => {
                        log::warn!("failed to read {}: {error:#}", path.display());
                        explorer.expanded.remove(&path);
                    }
                }
                explorer.rebuild_visible();
                cx.notify();
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while loading a folder: {error:#}");
            }
        })
        .detach();
    }

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
        } else {
            self.load_directory(path, cx);
        }
    }

    pub(super) fn toggle_root(&mut self, cx: &mut Context<Self>) {
        self.root_expanded = !self.root_expanded;
        self.rebuild_visible();
        cx.notify();
    }

    pub(super) fn refresh(&mut self, _: &RefreshExplorer, _: &mut Window, cx: &mut Context<Self>) {
        let mut folders: Vec<PathBuf> = self.expanded.iter().cloned().collect();
        folders.push(self.root.clone());
        self.children.clear();
        for folder in folders {
            self.load_directory(folder, cx);
        }
        self.project
            .update(cx, |project, cx| project.refresh_git(cx));
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

    fn rebuild_visible(&mut self) {
        let mut visible = Vec::new();
        if self.root_expanded {
            self.push_children(&self.root, 0, &mut visible);
        }
        self.visible = visible;
    }

    fn push_children(&self, path: &Path, depth: usize, visible: &mut Vec<VisibleEntry>) {
        let Some(children) = self.children.get(path) else {
            return;
        };
        for child in children {
            visible.push(VisibleEntry {
                path: child.path.clone(),
                name: child.name.clone(),
                depth,
                is_dir: child.is_dir,
            });
            if child.is_dir && self.expanded.contains(&child.path) {
                self.push_children(&child.path, depth + 1, visible);
            }
        }
    }
}
