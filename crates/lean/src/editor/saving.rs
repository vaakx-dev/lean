use std::io;
use std::path::Path;

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
        if let Err(error) = write_file(&file.path, content.as_bytes(), file.deleted) {
            log::error!("failed to save {}: {error}", file.path.display());
            window.push_notification(
                format!("Failed to save {}: {error}", file.path.display()),
                cx,
            );
            return false;
        }
        file.dirty = false;
        if file.deleted {
            file.deleted = false;
            cx.emit(EditorAreaEvent::FileRecreated(file.path.clone()));
        }
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
        true
    }
}

fn write_file(path: &Path, content: &[u8], recreate_folder: bool) -> io::Result<()> {
    if recreate_folder && let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    std::fs::write(path, content)
}
