mod notices;
mod outcomes;

use std::path::{Path, PathBuf};

use gpui_kit::*;

use super::EditorArea;
use super::disk_io::{DiskFile, DiskStamp, ReadError, read_changed_file};
use crate::watcher::FsChanges;

pub(super) use notices::dismiss_disk_notice;

struct DiskCheck {
    id: u64,
    path: PathBuf,
    known: Option<DiskStamp>,
    force: bool,
}

impl EditorArea {
    pub(crate) fn apply_fs_changes(
        &mut self,
        changes: &FsChanges,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for (from, to) in &changes.renames {
            self.follow_rename(from, to, cx);
        }
        let mut checks = Vec::new();
        for file in self.files.iter_mut().filter(|file| file.is_file()) {
            let changed = file
                .path
                .ancestors()
                .any(|path| changes.paths.contains(path));
            let ignored = file
                .path
                .ancestors()
                .any(|path| changes.ignored.contains(path));
            if !(changed || ignored || changes.rescan) {
                continue;
            }
            if file.disk.is_saving() {
                file.disk.changed_while_saving = true;
                continue;
            }
            checks.push(DiskCheck {
                id: file.id,
                path: file.path.clone(),
                known: if changed {
                    file.disk.written
                } else {
                    file.disk.stamp
                },
                force: false,
            });
        }
        for check in checks {
            self.check_disk(check, window, cx);
        }
    }

    pub(super) fn revert(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(file) = self.files.get(index).filter(|file| file.is_file()) else {
            return;
        };
        let check = DiskCheck {
            id: file.id,
            path: file.path.clone(),
            known: None,
            force: true,
        };
        self.check_disk(check, window, cx);
    }

    pub(super) fn revert_file(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.index_of_id(id) {
            self.revert(index, window, cx);
        }
    }

    pub(super) fn recheck_disk(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get(index).filter(|file| file.is_file()) else {
            return;
        };
        let check = DiskCheck {
            id: file.id,
            path: file.path.clone(),
            known: file.disk.written,
            force: false,
        };
        self.check_disk(check, window, cx);
    }

    fn follow_rename(&mut self, from: &Path, to: &Path, cx: &mut Context<Self>) {
        let is_open = self
            .files
            .iter()
            .any(|file| file.is_file() && file.path.starts_with(from));
        if is_open && !from.exists() {
            self.rename_files(from, to, cx);
        }
    }

    fn check_disk(&mut self, check: DiskCheck, window: &mut Window, cx: &mut Context<Self>) {
        let read = cx.background_spawn({
            let path = check.path.clone();
            let known = check.known;
            async move { read_changed_file(&path, known) }
        });
        let id = check.id;
        let task = cx.spawn_in(window, async move |area, cx| {
            let result = read.await;
            let update = area.update_in(cx, |area, window, cx| {
                area.apply_disk_read(check, result, window, cx)
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while checking a file on disk: {error:#}");
            }
        });
        if let Some(index) = self.index_of_id(id) {
            self.files[index].disk._check = task;
        }
    }

    fn apply_disk_read(
        &mut self,
        check: DiskCheck,
        result: Result<Option<DiskFile>, ReadError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.index_of_id(check.id) else {
            return;
        };
        let file = &mut self.files[index];
        if file.path != check.path {
            return;
        }
        if file.disk.is_saving() {
            file.disk.changed_while_saving = true;
            return;
        }
        let saved_hash = file.disk.saved_hash;
        let keeps_edits = file.dirty || file.view.running_work(cx).is_some();
        match result {
            Ok(None) => {}
            Ok(Some(disk)) if check.force => {
                self.reload(index, disk, window, cx);
            }
            Ok(Some(disk)) if disk.hash == saved_hash => {
                self.mark_unchanged(index, disk, window, cx)
            }
            Ok(Some(_)) if keeps_edits => self.mark_conflict(index, window, cx),
            Ok(Some(disk)) => self.apply_external_change(index, disk, window, cx),
            Err(ReadError::NotFound) => self.mark_deleted(index, cx),
            Err(error) => log::debug!("failed to check {}: {error}", check.path.display()),
        }
    }
}
