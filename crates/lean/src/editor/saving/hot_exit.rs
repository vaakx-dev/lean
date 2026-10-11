mod store;

use std::path::Path;
use std::time::Duration;

use gpui_kit::*;

use super::super::EditorArea;
use super::super::open_file::OpenFile;
use super::super::opening::Placement;
use crate::files::settings::HOT_EXIT;
use store::BackupStore;

pub(in crate::editor) use store::BackupFile;

const BACKUP_DELAY: Duration = Duration::from_secs(1);

#[derive(Default)]
pub(in crate::editor) struct HotExit {
    store: Option<BackupStore>,
    ready: bool,
    stored: bool,
    kept: Vec<(u64, u64)>,
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
        let stored = store.as_ref().is_some_and(BackupStore::exists);
        self.hot_exit = HotExit {
            store: store.clone(),
            stored,
            ..HotExit::default()
        };
        let read =
            cx.background_spawn(
                async move { store.filter(|_| stored).and_then(|store| store.read()) },
            );
        cx.spawn_in(window, async move |area, cx| {
            let backup = read.await;
            area.update_in(cx, |area, window, cx| {
                area.hot_exit.ready = true;
                match backup {
                    Some(backup) if HOT_EXIT.get(cx) => {
                        for file in backup.files {
                            let path = file.path.clone();
                            area.open_placed(path, Placement::Restore(file), window, cx);
                        }
                    }
                    _ => area.back_up_unsaved(false, cx),
                }
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn schedule_backup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let kept: Vec<(u64, u64)> = self
            .files
            .iter()
            .filter(|file| file.kept_on_exit(cx))
            .map(|file| (file.id, file.version))
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
        let Some(store) = &self.hot_exit.store else {
            return;
        };
        let files = self.unsaved_backups(cx);
        if files.is_empty() && !self.hot_exit.stored {
            return;
        }
        self.hot_exit.stored = !files.is_empty();
        if now {
            store.save_now(files);
        } else {
            store.save(files, cx);
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

    fn unsaved_backups(&self, cx: &App) -> Vec<BackupFile> {
        if !HOT_EXIT.get(cx) {
            return Vec::new();
        }
        self.files
            .iter()
            .filter(|file| file.is_file() && file.dirty)
            .filter_map(|file| {
                Some(BackupFile {
                    path: file.path.clone(),
                    base_hash: file.disk.saved_hash,
                    text: file.view.unsaved_text(cx)?,
                })
            })
            .collect()
    }
}

impl OpenFile {
    pub(in crate::editor) fn kept_on_exit(&self, cx: &App) -> bool {
        HOT_EXIT.get(cx) && self.is_file() && self.dirty && self.view.unsaved_text(cx).is_some()
    }
}
