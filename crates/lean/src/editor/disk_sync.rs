use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::disk_io::{DiskFile, DiskStamp, ReadError, read_changed_file};
use super::{EditorArea, EditorAreaEvent};
use crate::watcher::FsChanges;

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
        let checks: Vec<DiskCheck> = self
            .files
            .iter()
            .filter(|file| file.is_file() && !file.disk.saving)
            .filter_map(|file| {
                let changed = file
                    .path
                    .ancestors()
                    .any(|path| changes.paths.contains(path));
                let ignored = file
                    .path
                    .ancestors()
                    .any(|path| changes.ignored.contains(path));
                (changed || ignored || changes.rescan).then(|| DiskCheck {
                    id: file.id,
                    path: file.path.clone(),
                    known: if changed { None } else { file.disk.stamp },
                    force: false,
                })
            })
            .collect();
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
        let file = &self.files[index];
        if file.path != check.path || file.disk.saving {
            return;
        }
        let saved_hash = file.disk.saved_hash;
        let keeps_edits = file.dirty || file.view.running_work(cx).is_some();
        match result {
            Ok(None) => {}
            Ok(Some(disk)) if check.force => self.reload(index, disk, window, cx),
            Ok(Some(disk)) if disk.hash == saved_hash => self.mark_unchanged(index, disk, cx),
            Ok(Some(_)) if keeps_edits => self.mark_conflict(index, window, cx),
            Ok(Some(disk)) => self.reload(index, disk, window, cx),
            Err(ReadError::NotFound) => self.mark_deleted(index, cx),
            Err(error) => log::debug!("failed to check {}: {error}", check.path.display()),
        }
    }

    fn mark_deleted(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.disk.stamp = None;
        if !file.deleted {
            file.deleted = true;
            cx.notify();
        }
    }

    fn mark_unchanged(&mut self, index: usize, disk: DiskFile, cx: &mut Context<Self>) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.disk.stamp = Some(disk.stamp);
        if file.deleted || file.disk.conflict {
            file.deleted = false;
            file.disk.conflict = false;
            cx.notify();
        }
    }

    fn mark_conflict(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.deleted = false;
        if !file.disk.conflict {
            file.disk.conflict = true;
            let message = format!("{} changed on disk", file.name);
            window.push_notification(message, cx);
        }
        cx.notify();
    }

    fn reload(
        &mut self,
        index: usize,
        disk: DiskFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        let had_focus = file.view.focus_handle(cx).contains_focused(window, cx);
        if let Err(error) = file.view.reload(disk.bytes, window, cx) {
            log::warn!("failed to reload {}: {error:#}", file.path.display());
            let message = format!("Can't reload {}: {error:#}", file.name);
            window.push_notification(message, cx);
            return;
        }
        if had_focus {
            window.focus(&file.view.focus_handle(cx), cx);
        }
        file.disk.reloaded(disk.hash, disk.stamp);
        file.dirty = false;
        file.deleted = false;
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
    }
}
