mod store;
mod writer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::*;

use super::super::EditorArea;
use super::super::open_file::OpenFile;
use super::super::opening::Placement;
use crate::files::settings::HOT_EXIT;
use store::{BackupStore, read_backup};
use writer::{BackupChange, write_later, write_now};

pub(in crate::editor) use store::BackupFile;

const BACKUP_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
struct BackupKey {
    id: u64,
    version: u64,
    base_hash: u64,
}

impl BackupKey {
    fn of(file: &OpenFile) -> Self {
        Self {
            id: file.id,
            version: file.version,
            base_hash: file.disk.saved_hash,
        }
    }
}

#[derive(Default)]
pub(in crate::editor) struct HotExit {
    store: Option<BackupStore>,
    ready: bool,
    stored: HashMap<PathBuf, Option<BackupKey>>,
    kept: Vec<BackupKey>,
    _pending: Option<Task<()>>,
}

impl EditorArea {
    pub(super) fn start_hot_exit(
        &mut self,
        project_root: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let store = BackupStore::for_project(project_root);
        self.hot_exit = HotExit {
            store: store.clone(),
            ..HotExit::default()
        };
        let restore = HOT_EXIT.get(cx);
        let read = cx.background_spawn(async move {
            let files = store.map(|store| store.stored_files()).unwrap_or_default();
            let backups: Vec<BackupFile> = if restore {
                files.iter().filter_map(|file| read_backup(file)).collect()
            } else {
                Vec::new()
            };
            (files, backups)
        });
        cx.spawn_in(window, async move |area, cx| {
            let (files, backups) = read.await;
            area.update_in(cx, |area, window, cx| {
                area.hot_exit.ready = true;
                area.hot_exit.stored = files.into_iter().map(|file| (file, None)).collect();
                if backups.is_empty() || !HOT_EXIT.get(cx) {
                    area.back_up_unsaved(false, cx);
                    return;
                }
                for backup in backups {
                    let path = backup.path.clone();
                    area.open_placed(path, Placement::Restore(backup), window, cx)
                        .detach();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn schedule_backup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let enabled = HOT_EXIT.get(cx);
        let kept: Vec<BackupKey> = self
            .files
            .iter()
            .filter(|file| enabled && file.is_file() && file.dirty)
            .map(BackupKey::of)
            .collect();
        if self.hot_exit.kept == kept {
            return;
        }
        self.hot_exit.kept = kept;
        self.hot_exit._pending = Some(cx.spawn_in(window, async move |area, cx| {
            cx.background_executor().timer(BACKUP_DELAY).await;
            area.update(cx, |area, cx| area.back_up_unsaved(false, cx))
                .ok();
        }));
    }

    pub(super) fn back_up_unsaved(&mut self, now: bool, cx: &App) {
        if !self.hot_exit.ready {
            self.hot_exit.kept.clear();
            return;
        }
        let Some(store) = self.hot_exit.store.clone() else {
            return;
        };
        let changes = self.backup_changes(&store, cx);
        if changes.is_empty() {
            return;
        }
        if now {
            write_now(changes);
        } else {
            write_later(changes, cx);
        }
    }

    pub(in crate::editor) fn apply_backup(
        &mut self,
        index: usize,
        backup: BackupFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get_mut(index).filter(|file| !file.dirty) else {
            return;
        };
        file.view.restore_unsaved(backup.text, window, cx);
        file.dirty = file.view.differs_from_saved(cx);
        if file.dirty && file.disk.saved_hash != backup.base_hash {
            file.disk.conflict = true;
        }
        cx.notify();
    }

    fn backup_changes(&mut self, store: &BackupStore, cx: &App) -> Vec<BackupChange> {
        let enabled = HOT_EXIT.get(cx);
        let mut stored = HashMap::new();
        let mut changes = Vec::new();
        for file in &self.files {
            if !(enabled && file.is_file() && file.dirty) {
                continue;
            }
            let backup_file = store.file_for(&file.path);
            let key = Some(BackupKey::of(file));
            if self.hot_exit.stored.get(&backup_file) != Some(&key) {
                let Some(text) = file.view.unsaved_text(cx) else {
                    continue;
                };
                let backup = BackupFile {
                    path: file.path.clone(),
                    base_hash: file.disk.saved_hash,
                    text,
                };
                changes.push((backup_file.clone(), Some(backup)));
            }
            stored.insert(backup_file, key);
        }
        for backup_file in self.hot_exit.stored.keys() {
            if !stored.contains_key(backup_file) {
                changes.push((backup_file.clone(), None));
            }
        }
        self.hot_exit.stored = stored;
        changes
    }
}

impl OpenFile {
    pub(in crate::editor) fn kept_on_exit(&self, cx: &App) -> bool {
        HOT_EXIT.get(cx) && self.is_file() && self.dirty && self.view.unsaved_text(cx).is_some()
    }
}
