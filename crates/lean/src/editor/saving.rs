use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::{EditorArea, EditorAreaEvent};

impl EditorArea {
    pub(crate) fn save_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active {
            self.save(active, window, cx);
        }
    }

    pub(super) fn save(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(file) = self.files.get_mut(index) else {
            return false;
        };
        let content = file.editor.read(cx).value();
        if let Err(error) = std::fs::write(&file.path, content.as_bytes()) {
            log::error!("failed to save {}: {error}", file.path.display());
            window.push_notification(
                format!("Failed to save {}: {error}", file.path.display()),
                cx,
            );
            return false;
        }
        file.dirty = false;
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
        true
    }
}
