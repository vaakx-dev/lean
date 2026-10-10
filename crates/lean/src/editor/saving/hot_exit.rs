mod restore;
mod store;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::*;

use super::super::EditorArea;
use crate::files::settings::HOT_EXIT;
use store::{BackupFile, BackupStore};

const BACKUP_DELAY: Duration = Duration::from_secs(1);

pub(super) type SharedHotExit = Rc<RefCell<HotExit>>;

pub(super) struct HotExit {
    store: Option<BackupStore>,
    ready: bool,
    stored: bool,
    kept: Vec<(u64, u64)>,
    _pending: Task<()>,
}

impl HotExit {
    pub(super) fn start(
        project_root: &Path,
        window: &mut Window,
        cx: &mut Context<EditorArea>,
    ) -> SharedHotExit {
        let store = BackupStore::for_project(project_root);
        let stored = store.as_ref().is_some_and(BackupStore::exists);
        let hot_exit = Rc::new(RefCell::new(Self {
            store: store.clone(),
            ready: false,
            stored,
            kept: Vec::new(),
            _pending: Task::ready(()),
        }));
        let read =
            cx.background_spawn(
                async move { store.filter(|_| stored).and_then(|store| store.read()) },
            );
        let shared = hot_exit.clone();
        cx.spawn_in(window, async move |area, cx| {
            let backup = read.await;
            let update = area.update_in(cx, |area, window, cx| {
                shared.borrow_mut().ready = true;
                match backup {
                    Some(backup) if HOT_EXIT.get(cx) => {
                        area.restore_backup(backup.files, window, cx)
                    }
                    _ => shared.borrow_mut().write(area, cx, false),
                }
            });
            if let Err(error) = update {
                log::debug!("editor area dropped before restoring unsaved files: {error:#}");
            }
        })
        .detach();
        hot_exit
    }

    pub(super) fn schedule(
        this: &SharedHotExit,
        area: &mut EditorArea,
        window: &mut Window,
        cx: &mut Context<EditorArea>,
    ) {
        let kept = area.refresh_kept_on_exit(cx);
        if this.borrow().kept == kept {
            return;
        }
        let shared = this.clone();
        let pending = cx.spawn_in(window, async move |area, cx| {
            cx.background_executor().timer(BACKUP_DELAY).await;
            let update = area.update(cx, |area, cx| shared.borrow_mut().write(area, cx, false));
            if let Err(error) = update {
                log::debug!("editor area dropped before backing up unsaved files: {error:#}");
            }
        });
        let mut hot_exit = this.borrow_mut();
        hot_exit.kept = kept;
        hot_exit._pending = pending;
    }

    pub(super) fn write(&mut self, area: &EditorArea, cx: &App, now: bool) {
        if !self.ready {
            self.kept.clear();
            return;
        }
        let Some(store) = &self.store else {
            return;
        };
        let files = if HOT_EXIT.get(cx) {
            area.unsaved_backups(cx)
        } else {
            Vec::new()
        };
        if files.is_empty() && !self.stored {
            return;
        }
        self.stored = !files.is_empty();
        if now {
            store.save_now(files);
        } else {
            store.save(files, cx);
        }
    }
}

impl EditorArea {
    fn refresh_kept_on_exit(&mut self, cx: &App) -> Vec<(u64, u64)> {
        let enabled = HOT_EXIT.get(cx);
        let mut kept = Vec::new();
        for file in &mut self.files {
            file.disk.kept_on_exit =
                enabled && file.is_file() && file.dirty && file.view.unsaved_text(cx).is_some();
            if file.disk.kept_on_exit {
                kept.push((file.id, file.version));
            }
        }
        kept
    }

    fn unsaved_backups(&self, cx: &App) -> Vec<BackupFile> {
        self.files
            .iter()
            .filter(|file| file.dirty && file.disk.kept_on_exit)
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
