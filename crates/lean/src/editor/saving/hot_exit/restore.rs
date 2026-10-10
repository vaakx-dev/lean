use std::fmt::Display;
use std::path::Path;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::store::BackupFile;
use crate::editor::EditorArea;
use crate::editor::disk_io::{DiskFile, ReadError, read_file};
use crate::editor::open_file::OpenFile;
use crate::files::file_name;
use crate::plugins::{FileViewer, viewer_for};

impl EditorArea {
    pub(super) fn restore_backup(
        &mut self,
        files: Vec<BackupFile>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for backup in files {
            self.restore_file(backup, window, cx);
        }
    }

    fn restore_file(&mut self, backup: BackupFile, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.index_of(&backup.path) {
            self.apply_backup(index, backup, window, cx);
            return;
        }
        let Some(viewer) = viewer_for(&backup.path, cx) else {
            report_restore_error(&backup.path, &"no viewer can open it", window, cx);
            return;
        };
        let read = cx.background_spawn({
            let path = backup.path.clone();
            async move { read_file(&path) }
        });
        cx.spawn_in(window, async move |area, cx| {
            let disk = read.await;
            let update = area.update_in(cx, |area, window, cx| {
                area.open_backup(backup, viewer, disk, window, cx)
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while restoring an unsaved file: {error:#}");
            }
        })
        .detach();
    }

    fn open_backup(
        &mut self,
        backup: BackupFile,
        viewer: FileViewer,
        disk: Result<DiskFile, ReadError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of(&backup.path) {
            self.apply_backup(index, backup, window, cx);
            return;
        }
        let disk = match disk {
            Ok(disk) => disk,
            Err(error) => {
                report_restore_error(&backup.path, &error, window, cx);
                return;
            }
        };
        let path = backup.path.clone();
        let toolchains = self.toolchains.clone();
        let file = match OpenFile::open(path, viewer, disk, toolchains, window, cx) {
            Ok(file) => file,
            Err(error) => {
                report_restore_error(&backup.path, &format!("{error:#}"), window, cx);
                return;
            }
        };
        let index = self.insert(file, cx);
        if self.active.is_none() {
            self.activate(index, window, cx);
        }
        self.apply_backup(index, backup, window, cx);
    }

    fn apply_backup(
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
}

fn report_restore_error(path: &Path, error: &dyn Display, window: &mut Window, cx: &mut App) {
    log::warn!(
        "failed to restore unsaved changes to {}: {error}",
        path.display()
    );
    let message = format!(
        "Couldn't restore unsaved changes to {}: {error}",
        file_name(path)
    );
    window.push_notification(Notification::warning(message).autohide(false), cx);
}
