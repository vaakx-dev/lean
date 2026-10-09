use std::path::Path;

use gpui_kit::*;

use super::{EditorArea, EditorAreaEvent};

impl EditorArea {
    pub(crate) fn rename_files(&mut self, from: &Path, to: &Path, cx: &mut Context<Self>) {
        let mut renamed = false;
        for file in self.files.iter_mut().filter(|file| file.is_file()) {
            let Ok(suffix) = file.path.strip_prefix(from) else {
                continue;
            };
            let path = if suffix.as_os_str().is_empty() {
                to.to_path_buf()
            } else {
                to.join(suffix)
            };
            file.set_path(path, cx);
            renamed = true;
        }
        if renamed {
            cx.emit(EditorAreaEvent::OpenFilesChanged);
            cx.emit(EditorAreaEvent::ActiveEditorChanged);
            cx.notify();
        }
    }

    pub(crate) fn close_deleted_files(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut changed = false;
        for index in (0..self.files.len()).rev() {
            let Some(file) = self.files.get_mut(index) else {
                continue;
            };
            if !file.is_file() || !file.path.starts_with(path) {
                continue;
            }
            changed = true;
            if file.dirty {
                file.deleted = true;
            } else {
                self.remove(index, window, cx);
            }
        }
        if changed {
            cx.emit(EditorAreaEvent::ActiveEditorChanged);
            cx.notify();
        }
    }
}
