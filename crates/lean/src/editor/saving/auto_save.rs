use std::path::Path;
use std::time::Duration;

use gpui_kit::*;

use crate::editor::EditorArea;
use crate::editor::open_file::OpenFile;
use crate::files::settings::{AUTO_SAVE, AUTO_SAVE_DELAY_MS, AutoSave, SaveFormat};

impl EditorArea {
    pub(super) fn schedule_auto_save(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if AUTO_SAVE.get(cx) != AutoSave::AfterDelay {
            return;
        }
        let Some(index) = self.index_of(path) else {
            return;
        };
        let delay = Duration::from_millis(AUTO_SAVE_DELAY_MS.get(cx).clamp(200, 10_000));
        let id = self.files[index].id;
        self.files[index].disk._auto_save = cx.spawn_in(window, async move |area, cx| {
            cx.background_executor().timer(delay).await;
            let update = area.update_in(cx, |area, window, cx| {
                if AUTO_SAVE.get(cx) == AutoSave::AfterDelay {
                    let format = SaveFormat::from_settings(cx).without_text_changes();
                    area.auto_save(id, format, window, cx);
                }
            });
            if let Err(error) = update {
                log::debug!("editor area dropped before an auto save: {error:#}");
            }
        });
    }

    pub(super) fn auto_save_on_focus_change(
        &mut self,
        id: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if AUTO_SAVE.get(cx) == AutoSave::OnFocusChange {
            self.auto_save(id, SaveFormat::from_settings(cx), window, cx);
        }
    }

    pub(super) fn auto_save_active_on_focus_change(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.active_file().map(|file| file.id) {
            self.auto_save_on_focus_change(id, window, cx);
        }
    }

    pub(super) fn auto_save_all_on_focus_change(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if AUTO_SAVE.get(cx) != AutoSave::OnFocusChange {
            return;
        }
        let ids: Vec<u64> = self
            .files
            .iter()
            .filter(|file| can_auto_save(file))
            .map(|file| file.id)
            .collect();
        for id in ids {
            self.auto_save(id, SaveFormat::from_settings(cx), window, cx);
        }
    }

    fn auto_save(
        &mut self,
        id: u64,
        format: SaveFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.index_of_id(id) else {
            return;
        };
        if can_auto_save(&self.files[index]) {
            drop(self.write(index, true, format, window, cx));
        }
    }
}

fn can_auto_save(file: &OpenFile) -> bool {
    file.is_file() && file.dirty && !file.deleted && !file.disk.conflict && !file.disk.is_saving()
}
