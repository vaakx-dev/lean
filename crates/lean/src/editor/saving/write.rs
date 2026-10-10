use std::fmt::Display;
use std::io;
use std::path::Path;

use futures::FutureExt as _;
use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::super::disk_state::{SaveInFlight, SaveOutcome};
use super::super::disk_sync::dismiss_disk_notice;
use super::super::{EditorArea, EditorAreaEvent};
use super::snapshot::{DiskWrite, ExpectedDisk, write_snapshot};
use crate::files::settings::SaveFormat;

struct SaveRequest {
    id: u64,
    version: u64,
    on_saved: Option<Box<dyn FnOnce(&mut App)>>,
}

impl EditorArea {
    pub(super) fn write(
        &mut self,
        index: usize,
        check_disk: bool,
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
        let expected = (check_disk && !file.deleted).then_some(ExpectedDisk {
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
                .unwrap_or_else(|error| {
                    log::debug!("editor area dropped while saving a file: {error:#}");
                    SaveOutcome::Failed
                })
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
        let Some(index) = self.index_of_id(request.id) else {
            return match result {
                Ok(DiskWrite::Written { .. }) => SaveOutcome::Saved,
                Ok(DiskWrite::ChangedOnDisk) => SaveOutcome::ChangedOnDisk,
                Err(_) => SaveOutcome::Failed,
            };
        };
        let file = &mut self.files[index];
        file.disk.saving = None;
        let recheck = std::mem::take(&mut file.disk.changed_while_saving);
        let outcome = match result {
            Ok(DiskWrite::Written { stamp, hash }) => {
                if file.disk.conflict {
                    dismiss_disk_notice(file.id, window, cx);
                }
                file.disk.saved(hash, stamp);
                self.apply_saved(index, request, cx);
                SaveOutcome::Saved
            }
            Ok(DiskWrite::ChangedOnDisk) => {
                self.mark_conflict(index, window, cx);
                SaveOutcome::ChangedOnDisk
            }
            Err(error) => {
                report_save_error(&file.path, &error, window, cx);
                cx.notify();
                SaveOutcome::Failed
            }
        };
        if recheck && outcome != SaveOutcome::ChangedOnDisk {
            self.recheck_disk(index, window, cx);
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

fn finished(outcome: SaveOutcome) -> SaveInFlight {
    Task::ready(outcome).shared()
}

fn report_save_error(path: &Path, error: &dyn Display, window: &mut Window, cx: &mut App) {
    log::error!("failed to save {}: {error}", path.display());
    window.push_notification(format!("Failed to save {}: {error}", path.display()), cx);
}
