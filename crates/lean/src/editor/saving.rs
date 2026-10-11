mod auto_save;
mod conflict;
mod format;
mod hot_exit;
mod services;
mod write;

use gpui_kit::*;

use super::EditorArea;
use super::disk_state::{SaveInFlight, SaveOutcome};
use crate::files::settings::SaveFormat;

pub(super) use format::format_for_save;
pub(super) use hot_exit::{BackupFile, HotExit};

impl EditorArea {
    pub(crate) fn save_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active {
            self.save(active, window, cx).detach();
        }
    }

    pub(crate) fn save_unsaved(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let ids: Vec<u64> = self
            .files
            .iter()
            .filter(|file| file.dirty || file.disk.saving.is_some())
            .map(|file| file.id)
            .collect();
        cx.spawn_in(window, async move |area, cx| {
            for id in ids {
                let save = area.update_in(cx, |area, window, cx| {
                    area.index_of_id(id)
                        .map(|index| area.save(index, window, cx))
                });
                if !settle(save).await {
                    return false;
                }
            }
            true
        })
    }

    pub(super) fn save(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let Some(file) = self.files.get(index) else {
            return Task::ready(false);
        };
        if !file.is_file() {
            return Task::ready(true);
        }
        let id = file.id;
        if let Some(in_flight) = file.disk.saving.clone() {
            return self.save_after(id, in_flight, window, cx);
        }
        if !file.dirty && !file.deleted {
            return Task::ready(true);
        }
        if file.disk.conflict {
            return self.resolve_conflict(index, window, cx);
        }
        let write = self.write(index, SaveFormat::from_settings(cx), window, cx);
        self.resolve_after_write(id, write, window, cx)
    }

    fn save_after(
        &mut self,
        id: u64,
        in_flight: SaveInFlight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        cx.spawn_in(window, async move |area, cx| {
            if in_flight.await != SaveOutcome::Saved {
                return false;
            }
            let save = area.update_in(cx, |area, window, cx| {
                area.index_of_id(id)
                    .map(|index| area.save(index, window, cx))
            });
            settle(save).await
        })
    }

    fn resolve_after_write(
        &mut self,
        id: u64,
        write: SaveInFlight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        cx.spawn_in(window, async move |area, cx| match write.await {
            SaveOutcome::Saved => true,
            SaveOutcome::Failed => false,
            SaveOutcome::ChangedOnDisk => {
                let resolve = area.update_in(cx, |area, window, cx| {
                    area.index_of_id(id)
                        .map(|index| area.resolve_conflict(index, window, cx))
                });
                settle(resolve).await
            }
        })
    }
}

async fn settle(task: anyhow::Result<Option<Task<bool>>>) -> bool {
    match task {
        Ok(Some(task)) => task.await,
        Ok(None) => true,
        Err(_) => false,
    }
}
