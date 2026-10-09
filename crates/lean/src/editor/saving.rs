mod snapshot;

use std::fmt::Display;
use std::io;
use std::path::Path;

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::{EditorArea, EditorAreaEvent};
use snapshot::{Written, write_snapshot};

const OVERWRITE_ANSWER: usize = 0;
const REVERT_ANSWER: usize = 1;

struct SaveRequest {
    id: u64,
    version: u64,
    on_saved: Option<Box<dyn FnOnce(&mut App)>>,
}

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
            .filter(|file| file.dirty)
            .map(|file| file.id)
            .collect();
        cx.spawn_in(window, async move |area, cx| {
            for id in ids {
                let save = area.update_in(cx, |area, window, cx| {
                    area.index_of_id(id)
                        .map(|index| area.save(index, window, cx))
                });
                let saved = match save {
                    Ok(Some(save)) => save.await,
                    Ok(None) => true,
                    Err(error) => {
                        log::debug!("editor area dropped while saving all files: {error:#}");
                        false
                    }
                };
                if !saved {
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
        if file.disk.saving {
            return Task::ready(false);
        }
        if file.disk.conflict {
            return self.resolve_conflict(index, window, cx);
        }
        self.write(index, window, cx)
    }

    fn resolve_conflict(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let Some(file) = self.files.get(index) else {
            return Task::ready(false);
        };
        let id = file.id;
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("{} has changed on disk.", file.name),
            Some("Overwrite it with your changes, or revert to the version on disk?"),
            &[
                PromptButton::ok("Overwrite"),
                PromptButton::new("Revert"),
                PromptButton::cancel("Cancel"),
            ],
            cx,
        );
        cx.spawn_in(window, async move |area, cx| {
            let answer = match answer.await {
                Ok(answer) => answer,
                Err(error) => {
                    log::debug!("the conflict prompt was dismissed: {error}");
                    return false;
                }
            };
            let write = area.update_in(cx, |area, window, cx| {
                let index = area.index_of_id(id)?;
                match answer {
                    OVERWRITE_ANSWER => Some(area.write(index, window, cx)),
                    REVERT_ANSWER => {
                        area.revert(index, window, cx);
                        None
                    }
                    _ => None,
                }
            });
            match write {
                Ok(Some(write)) => write.await,
                Ok(None) => false,
                Err(error) => {
                    log::debug!("editor area dropped while resolving a conflict: {error:#}");
                    false
                }
            }
        })
    }

    fn write(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> Task<bool> {
        let Some(file) = self.files.get_mut(index) else {
            return Task::ready(false);
        };
        let snapshot = match file.view.snapshot(cx) {
            None => return Task::ready(true),
            Some(Ok(snapshot)) => snapshot,
            Some(Err(error)) => {
                report_save_error(&file.path, &format!("{error:#}"), window, cx);
                return Task::ready(false);
            }
        };
        let request = SaveRequest {
            id: file.id,
            version: file.version,
            on_saved: snapshot.on_saved,
        };
        let encode = snapshot.encode;
        let recreate_folder = file.deleted;
        let path = file.path.clone();
        file.disk.saving = true;
        file.disk._check = Task::ready(());
        cx.notify();
        let write =
            cx.background_spawn(async move { write_snapshot(encode, &path, recreate_folder) });
        cx.spawn_in(window, async move |area, cx| {
            let result = write.await;
            area.update_in(cx, |area, window, cx| {
                area.finish_save(request, result, window, cx)
            })
            .unwrap_or_else(|error| {
                log::debug!("editor area dropped while saving a file: {error:#}");
                false
            })
        })
    }

    fn finish_save(
        &mut self,
        request: SaveRequest,
        result: io::Result<Written>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(index) = self.index_of_id(request.id) else {
            return result.is_ok();
        };
        let file = &mut self.files[index];
        file.disk.saving = false;
        let written = match result {
            Ok(written) => written,
            Err(error) => {
                report_save_error(&file.path, &error, window, cx);
                cx.notify();
                return false;
            }
        };
        file.disk.saved_hash = written.hash;
        file.disk.stamp = Some(written.stamp);
        file.disk.conflict = false;
        if let Some(on_saved) = request.on_saved {
            on_saved(cx);
        }
        file.dirty = file.version != request.version && file.view.differs_from_saved(cx);
        if std::mem::take(&mut file.deleted) {
            cx.emit(EditorAreaEvent::FileRecreated(file.path.clone()));
        }
        cx.emit(EditorAreaEvent::Saved);
        cx.notify();
        true
    }
}

fn report_save_error(path: &Path, error: &dyn Display, window: &mut Window, cx: &mut App) {
    log::error!("failed to save {}: {error}", path.display());
    window.push_notification(format!("Failed to save {}: {error}", path.display()), cx);
}
