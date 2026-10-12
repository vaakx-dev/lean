mod notices;
mod outcomes;

use std::path::{Path, PathBuf};

use gpui_kit::*;

use super::EditorArea;
use super::disk_io::{DiskFile, DiskStamp, ReadError, read_changed_file};
use super::text_file::discard_decoded;
use crate::plugins::viewer_for;
use crate::watcher::FsChanges;

pub(super) use notices::dismiss_disk_notice;

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
        for (index, file) in self.files.iter_mut().enumerate() {
            if !file.is_file() {
                continue;
            }
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
            if file.disk.saving.is_some() {
                file.disk.changed_while_saving = true;
                continue;
            }
            let known = if changed {
                file.disk.written
            } else {
                file.disk.stamp
            };
            checks.push((index, known));
        }
        for (index, known) in checks {
            self.check_disk(index, known, false, window, cx);
        }
    }

    pub(super) fn revert(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.check_disk(index, None, true, window, cx);
    }

    pub(super) fn check_disk(
        &mut self,
        index: usize,
        known: Option<DiskStamp>,
        force: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get_mut(index).filter(|file| file.is_file()) else {
            return;
        };
        let id = file.id;
        let path = file.path.clone();
        let saved_hash = file.disk.saved_hash;
        let prepare = if force || !file.dirty {
            viewer_for(&path, cx).and_then(|viewer| viewer.prepare)
        } else {
            None
        };
        let read = cx.background_spawn({
            let path = path.clone();
            async move {
                read_changed_file(&path, known).inspect(|disk| {
                    if let (Some(disk), Some(prepare)) = (disk, prepare)
                        && (force || disk.hash != saved_hash)
                    {
                        prepare(&path, &disk.bytes);
                    }
                })
            }
        });
        file.disk._check = cx.spawn_in(window, async move |area, cx| {
            let result = read.await;
            area.update_in(cx, |area, window, cx| {
                area.apply_disk_read(id, path.clone(), force, result, window, cx)
            })
            .ok();
            discard_decoded(&path);
        });
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

    fn apply_disk_read(
        &mut self,
        id: u64,
        path: PathBuf,
        force: bool,
        result: Result<Option<DiskFile>, ReadError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.index_of_id(id) else {
            return;
        };
        let file = &mut self.files[index];
        if file.path != path {
            return;
        }
        if file.disk.saving.is_some() {
            file.disk.changed_while_saving = true;
            return;
        }
        let saved_hash = file.disk.saved_hash;
        let keeps_edits = file.dirty || file.view.running_work(cx).is_some();
        match result {
            Ok(None) => {}
            Ok(Some(disk)) if force => {
                self.reload(index, disk, window, cx);
            }
            Ok(Some(disk)) if disk.hash == saved_hash => {
                self.mark_unchanged(index, disk, window, cx)
            }
            Ok(Some(_)) if keeps_edits => self.mark_conflict(index, false, window, cx),
            Ok(Some(disk)) => self.apply_external_change(index, disk, window, cx),
            Err(ReadError::NotFound) => self.mark_deleted(index, cx),
            Err(error) => log::debug!("failed to check {}: {error}", path.display()),
        }
    }
}
