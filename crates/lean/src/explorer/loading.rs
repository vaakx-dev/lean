mod folder_loads;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{Context, Window};

use super::Explorer;
use super::sorting::sort_entries;
use crate::files::file_name;
use crate::project::DirectoryEntry;

pub(super) use folder_loads::FolderLoads;

type FolderLoad = (PathBuf, u64);

impl Explorer {
    pub(super) fn load_directory(
        &mut self,
        path: PathBuf,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        self.load_directories(vec![path], window, cx);
    }

    pub(super) fn load_directory_and_reveal(
        &mut self,
        path: PathBuf,
        reveal: Option<PathBuf>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.loads.begin(&path, reveal);
        self.read_folders(vec![(path, id)], window, cx);
    }

    pub(super) fn load_directories(
        &mut self,
        folders: Vec<PathBuf>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if folders.is_empty() {
            return;
        }
        let loads: Vec<FolderLoad> = folders
            .into_iter()
            .map(|folder| {
                let id = self.loads.begin(&folder, None);
                (folder, id)
            })
            .collect();
        self.read_folders(loads, window, cx);
    }

    fn read_folders(&mut self, loads: Vec<FolderLoad>, window: &Window, cx: &mut Context<Self>) {
        let folders = loads.iter().map(|(folder, _)| folder.clone()).collect();
        let task = self.project.read(cx).read_directories(folders, cx);
        cx.spawn_in(window, async move |explorer, cx| {
            let results = task.await;
            explorer
                .update_in(cx, |explorer, window, cx| {
                    explorer.finish_loads(loads, results, window, cx)
                })
                .ok();
        })
        .detach();
    }

    fn finish_loads(
        &mut self,
        loads: Vec<FolderLoad>,
        results: Vec<Result<Vec<DirectoryEntry>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut finished = Vec::new();
        let mut vanished = Vec::new();
        let mut reveal = None;
        for ((folder, id), result) in loads.into_iter().zip(results) {
            let Some(load) = self.loads.finish(&folder, id) else {
                continue;
            };
            match result {
                Ok(entries) => vanished.extend(self.replace_children(folder.clone(), entries)),
                Err(error) => self.handle_load_error(&folder, &error, window, cx),
            }
            reveal = load.reveal.or(reveal);
            finished.push(folder);
        }
        if finished.is_empty() {
            return;
        }
        self.rebuild_visible();
        self.reselect_if_vanished(&vanished);
        if let Some(reveal) = reveal {
            self.selected = Some(reveal.clone());
            self.pending_reveal = Some(reveal);
        }
        self.scroll_to_pending_reveal();
        cx.notify();
        for folder in &finished {
            if self.unfolding.remove(folder) {
                self.unfold_single_child(folder, window, cx);
            }
        }
        if !vanished.is_empty() {
            self.cancel_orphaned_edit(window, cx);
        }
    }

    fn handle_load_error(
        &mut self,
        path: &Path,
        error: &anyhow::Error,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.unfolding.remove(path);
        let is_missing = error
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == io::ErrorKind::NotFound);
        self.forget_subtrees(&[path]);
        if is_missing {
            log::debug!("{} no longer exists", path.display());
        } else {
            log::warn!("failed to read {}: {error:#}", path.display());
            let message = format!("Can't open the folder \"{}\": {error}", file_name(path));
            window.push_notification(Notification::warning(message), cx);
        }
    }

    fn replace_children(
        &mut self,
        folder: PathBuf,
        mut entries: Vec<DirectoryEntry>,
    ) -> Vec<Arc<Path>> {
        sort_entries(&mut entries, self.options.sorting);
        let current: HashMap<&Path, bool> = entries
            .iter()
            .map(|entry| (&*entry.path, entry.is_dir))
            .collect();
        let mut vanished = Vec::new();
        let mut lost_folders = Vec::new();
        for old in self.children.get(&folder).into_iter().flatten() {
            let is_dir = current.get(&*old.path).copied();
            if is_dir.is_none() {
                vanished.push(old.path.clone());
            }
            if old.is_dir && is_dir != Some(true) {
                lost_folders.push(old.path.clone());
            }
        }
        self.forget_subtrees(&lost_folders);
        self.children.insert(folder, entries);
        vanished
    }

    pub(super) fn forget_subtrees(&mut self, folders: &[impl AsRef<Path>]) {
        if folders.is_empty() {
            return;
        }
        let is_inside = |path: &Path| folders.iter().any(|folder| path.starts_with(folder));
        self.children.retain(|folder, _| !is_inside(folder));
        self.expanded.retain(|folder| !is_inside(folder));
        self.loads.forget(is_inside);
    }

    fn reselect_if_vanished(&mut self, vanished: &[Arc<Path>]) {
        let Some(selected) = self.selected.as_ref() else {
            return;
        };
        if !vanished.iter().any(|path| selected.starts_with(path)) {
            return;
        }
        let visible: HashSet<&Path> = self
            .visible
            .iter()
            .filter(|entry| !entry.is_draft)
            .map(|entry| &*entry.path)
            .collect();
        let survivor = selected
            .ancestors()
            .skip(1)
            .find(|ancestor| visible.contains(ancestor))
            .map(Path::to_path_buf);
        self.selected = survivor;
    }

    fn cancel_orphaned_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let is_orphaned = self.editing.is_some()
            && !self
                .visible
                .iter()
                .any(|entry| self.edit_for(entry).is_some());
        if is_orphaned {
            self.cancel_edit(window, cx);
        }
    }
}
