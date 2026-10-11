use std::fmt::Display;
use std::io;
use std::path::Path;

use futures::FutureExt as _;
use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::super::disk_io::{DiskStamp, ReadError, read_changed_file, write_atomically};
use super::super::disk_state::{SaveInFlight, SaveOutcome};
use super::super::disk_sync::dismiss_disk_notice;
use super::super::file_text::content_hash;
use super::super::{EditorArea, EditorAreaEvent};
use crate::files::settings::SaveFormat;

struct SaveRequest {
    id: u64,
    version: u64,
    on_saved: Option<Box<dyn FnOnce(&mut App)>>,
}

struct ExpectedDisk {
    stamp: Option<DiskStamp>,
    hash: u64,
}

enum DiskWrite {
    Written { stamp: DiskStamp, hash: u64 },
    ChangedOnDisk,
}

impl EditorArea {
    pub(super) fn write(
        &mut self,
        index: usize,
        format: SaveFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SaveInFlight {
        let Some(file) = self.files.get_mut(index) else {
            return finished(SaveOutcome::Failed);
        };
        file.view.prepare_save(&format, window, cx);
        let snapshot = match file.view.snapshot(cx) {
            None => return finished(SaveOutcome::Saved),
            Some(Ok(snapshot)) => snapshot,
            Some(Err(error)) => {
                report_save_error(&file.path, &format!("{error:#}"), window, cx);
                return finished(SaveOutcome::Failed);
            }
        };
        let request = SaveRequest {
            id: file.id,
            version: file.version,
            on_saved: snapshot.on_saved,
        };
        let encode = snapshot.encode;
        let recreate_folder = file.deleted;
        let expected = (!file.disk.conflict && !file.deleted).then_some(ExpectedDisk {
            stamp: file.disk.stamp,
            hash: file.disk.saved_hash,
        });
        let path = file.path.clone();
        let write =
            cx.background_spawn(
                async move { write_snapshot(encode, &path, recreate_folder, expected) },
            );
        let job = cx
            .spawn_in(window, async move |area, cx| {
                let result = write.await;
                area.update_in(cx, |area, window, cx| {
                    area.finish_save(request, result, window, cx)
                })
                .unwrap_or(SaveOutcome::Failed)
            })
            .shared();
        file.disk.saving = Some(job.clone());
        file.disk._check = Task::ready(());
        cx.notify();
        job
    }

    fn finish_save(
        &mut self,
        request: SaveRequest,
        result: io::Result<DiskWrite>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SaveOutcome {
        let outcome = match &result {
            Ok(DiskWrite::Written { .. }) => SaveOutcome::Saved,
            Ok(DiskWrite::ChangedOnDisk) => SaveOutcome::ChangedOnDisk,
            Err(_) => SaveOutcome::Failed,
        };
        let Some(index) = self.index_of_id(request.id) else {
            return outcome;
        };
        let file = &mut self.files[index];
        file.disk.saving = None;
        let recheck = std::mem::take(&mut file.disk.changed_while_saving);
        match result {
            Ok(DiskWrite::Written { stamp, hash }) => {
                if file.disk.conflict {
                    dismiss_disk_notice(file.id, window, cx);
                }
                file.disk.saved(hash, stamp);
                self.apply_saved(index, request, cx);
            }
            Ok(DiskWrite::ChangedOnDisk) => self.mark_conflict(index, false, window, cx),
            Err(error) => {
                report_save_error(&file.path, &error, window, cx);
                cx.notify();
            }
        }
        if recheck && outcome != SaveOutcome::ChangedOnDisk {
            let known = self.files[index].disk.written;
            self.check_disk(index, known, false, window, cx);
        }
        outcome
    }

    fn apply_saved(&mut self, index: usize, request: SaveRequest, cx: &mut Context<Self>) {
        let file = &mut self.files[index];
        if let Some(on_saved) = request.on_saved {
            on_saved(cx);
        }
        file.dirty = file.version != request.version && file.view.differs_from_saved(cx);
        if std::mem::take(&mut file.deleted) {
            cx.emit(EditorAreaEvent::FileRecreated(file.path.clone()));
        }
        cx.emit(EditorAreaEvent::Saved);
        cx.notify();
    }
}

fn write_snapshot(
    encode: Box<dyn FnOnce() -> Vec<u8> + Send>,
    path: &Path,
    recreate_folder: bool,
    expected: Option<ExpectedDisk>,
) -> io::Result<DiskWrite> {
    if let Some(expected) = expected
        && changed_on_disk(path, &expected)
    {
        return Ok(DiskWrite::ChangedOnDisk);
    }
    let bytes = encode();
    let hash = content_hash(&bytes);
    let stamp = write_atomically(path, &bytes, recreate_folder)?;
    Ok(DiskWrite::Written { stamp, hash })
}

fn changed_on_disk(path: &Path, expected: &ExpectedDisk) -> bool {
    match read_changed_file(path, expected.stamp) {
        Ok(None) | Err(ReadError::NotFound) => false,
        Ok(Some(disk)) => disk.hash != expected.hash,
        Err(error) => {
            log::debug!("can't check {} before saving: {error}", path.display());
            true
        }
    }
}

fn finished(outcome: SaveOutcome) -> SaveInFlight {
    Task::ready(outcome).shared()
}

fn report_save_error(path: &Path, error: &dyn Display, window: &mut Window, cx: &mut App) {
    log::error!("failed to save {}: {error}", path.display());
    window.push_notification(format!("Failed to save {}: {error}", path.display()), cx);
}
