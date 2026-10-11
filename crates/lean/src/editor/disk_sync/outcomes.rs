use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::super::disk_io::DiskFile;
use super::super::{EditorArea, EditorAreaEvent};
use super::notices;
use crate::files::settings::{EXTERNAL_CHANGE_NOTICE, EXTERNAL_CHANGES, ExternalChanges};

impl EditorArea {
    pub(in crate::editor) fn mark_conflict(
        &mut self,
        index: usize,
        offer_reload: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let area = cx.weak_entity();
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.deleted = false;
        file.disk.written = None;
        if !file.disk.conflict {
            file.disk.conflict = true;
            let notice = if offer_reload {
                notices::reload_offer(area, &file.name, file.id)
            } else {
                notices::changed_on_disk(&file.name, file.id)
            };
            window.push_notification(notice, cx);
        }
        cx.notify();
    }

    pub(super) fn apply_external_change(
        &mut self,
        index: usize,
        disk: DiskFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if EXTERNAL_CHANGES.get(cx) == ExternalChanges::Ask {
            self.mark_conflict(index, true, window, cx);
            return;
        }
        if self.reload(index, disk, window, cx) && EXTERNAL_CHANGE_NOTICE.get(cx) {
            let file = &self.files[index];
            window.push_notification(notices::reloaded(&file.name, file.id), cx);
        }
    }

    pub(super) fn mark_deleted(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.disk.stamp = None;
        file.disk.written = None;
        if !file.deleted {
            file.deleted = true;
            cx.notify();
        }
    }

    pub(super) fn mark_unchanged(
        &mut self,
        index: usize,
        disk: DiskFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get_mut(index) else {
            return;
        };
        file.disk.stamp = Some(disk.stamp);
        file.disk.written = None;
        if file.deleted || file.disk.conflict {
            file.deleted = false;
            file.disk.conflict = false;
            notices::dismiss_disk_notice(file.id, window, cx);
            cx.notify();
        }
    }

    pub(super) fn reload(
        &mut self,
        index: usize,
        disk: DiskFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(file) = self.files.get_mut(index) else {
            return false;
        };
        let had_focus = file.view.focus_handle(cx).contains_focused(window, cx);
        if let Err(error) = file.view.reload(disk.bytes, window, cx) {
            log::warn!("failed to reload {}: {error:#}", file.path.display());
            let message = format!("Can't reload {}: {error:#}", file.name);
            window.push_notification(message, cx);
            return false;
        }
        if had_focus {
            window.focus(&file.view.focus_handle(cx), cx);
        }
        if file.disk.conflict {
            notices::dismiss_disk_notice(file.id, window, cx);
        }
        file.disk.reloaded(disk.hash, disk.stamp);
        file.dirty = false;
        file.deleted = false;
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
        true
    }
}
