use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::{App, Context, Task, Window};

use super::Explorer;
use super::sorting::sort_entries;
use crate::files::file_name;
use crate::project::DirectoryEntry;

#[derive(Default)]
pub(super) struct FolderLoads {
    next_id: u64,
    pending: HashMap<PathBuf, PendingLoad>,
}

struct PendingLoad {
    id: u64,
    reveal: Option<PathBuf>,
}

impl FolderLoads {
    fn begin(&mut self, folder: &Path, reveal: Option<PathBuf>) -> u64 {
        self.next_id += 1;
        let reveal = reveal.or_else(|| {
            self.pending
                .remove(folder)
                .and_then(|pending| pending.reveal)
        });
        let id = self.next_id;
        self.pending
            .insert(folder.to_path_buf(), PendingLoad { id, reveal });
        id
    }

    fn finish(&mut self, folder: &Path, id: u64) -> Option<PendingLoad> {
        if self.pending.get(folder)?.id != id {
            return None;
        }
        self.pending.remove(folder)
    }

    fn forget_under(&mut self, path: &Path) {
        self.pending.retain(|folder, _| !folder.starts_with(path));
    }

    pub(super) fn is_idle(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Explorer {
    pub(super) fn load_directory(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.load_directory_and_reveal(path, None, cx);
    }

    pub(super) fn load_directory_and_reveal(
        &mut self,
        path: PathBuf,
        reveal: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        let (id, task) = self.start_load(&path, reveal, cx);
        cx.spawn(async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update(cx, |explorer, cx| {
                explorer.finish_load(path, id, result, cx);
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while loading a folder: {error:#}");
            }
        })
        .detach();
    }

    pub(super) fn reload_folder(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (id, task) = self.start_load(&path, None, cx);
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update_in(cx, |explorer, window, cx| {
                if explorer.finish_load(path, id, result, cx) {
                    explorer.cancel_orphaned_edit(window, cx);
                }
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while reloading a folder: {error:#}");
            }
        })
        .detach();
    }

    fn start_load(
        &mut self,
        path: &Path,
        reveal: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> (u64, Task<Result<Vec<DirectoryEntry>>>) {
        let id = self.loads.begin(path, reveal);
        let task = self.project.read(cx).read_directory(path, cx);
        (id, task)
    }

    fn finish_load(
        &mut self,
        path: PathBuf,
        id: u64,
        result: Result<Vec<DirectoryEntry>>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(load) = self.loads.finish(&path, id) else {
            return false;
        };
        let vanished = match result {
            Ok(entries) => self.replace_children(path.clone(), entries),
            Err(error) => {
                self.handle_load_error(&path, &error, cx);
                Vec::new()
            }
        };
        self.rebuild_visible();
        self.reselect_if_vanished(&vanished);
        if let Some(reveal) = load.reveal {
            self.selected = Some(reveal.clone());
            self.pending_reveal = Some(reveal);
        }
        self.scroll_to_pending_reveal();
        cx.notify();
        if self.unfolding.remove(&path) {
            self.unfold_single_child(&path, cx);
        }
        !vanished.is_empty()
    }

    fn handle_load_error(&mut self, path: &Path, error: &anyhow::Error, cx: &mut App) {
        self.unfolding.remove(path);
        let is_missing = error
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == io::ErrorKind::NotFound);
        self.forget_subtree(path);
        if is_missing {
            log::debug!("{} no longer exists", path.display());
        } else {
            log::warn!("failed to read {}: {error:#}", path.display());
            self.show_warning(
                format!("Can't open the folder \"{}\": {error}", file_name(path)),
                cx,
            );
        }
    }

    fn replace_children(
        &mut self,
        folder: PathBuf,
        mut entries: Vec<DirectoryEntry>,
    ) -> Vec<PathBuf> {
        sort_entries(&mut entries, self.options.sorting);
        let current: HashMap<&Path, bool> = entries
            .iter()
            .map(|entry| (entry.path.as_path(), entry.is_dir))
            .collect();
        let mut vanished = Vec::new();
        let mut lost_folders = Vec::new();
        for old in self.children.get(&folder).into_iter().flatten() {
            match current.get(old.path.as_path()).copied() {
                None => vanished.push(old.path.clone()),
                Some(false) if old.is_dir => lost_folders.push(old.path.clone()),
                Some(_) => {}
            }
        }
        for path in vanished.iter().chain(&lost_folders) {
            self.forget_subtree(path);
        }
        self.children.insert(folder, entries);
        vanished
    }

    fn forget_subtree(&mut self, path: &Path) {
        self.children.retain(|folder, _| !folder.starts_with(path));
        self.expanded.retain(|folder| !folder.starts_with(path));
        self.loads.forget_under(path);
    }

    fn reselect_if_vanished(&mut self, vanished: &[PathBuf]) {
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
            .map(|entry| entry.path.as_path())
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
