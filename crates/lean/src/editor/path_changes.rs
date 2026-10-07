use std::path::Path;

use gpui_kit::*;

use super::{EditorArea, EditorAreaEvent};

impl EditorArea {
    pub(crate) fn rename_files(&mut self, from: &Path, to: &Path, cx: &mut Context<Self>) {
        for file in &mut self.files {
            let Ok(suffix) = file.path.strip_prefix(from) else {
                continue;
            };
            let path = if suffix.as_os_str().is_empty() {
                to.to_path_buf()
            } else {
                to.join(suffix)
            };
            file.set_path(path, cx);
        }
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
    }

    pub(crate) fn close_deleted_files(&mut self, path: &Path, cx: &mut Context<Self>) {
        for index in (0..self.files.len()).rev() {
            let Some(file) = self.files.get_mut(index) else {
                continue;
            };
            if !file.path.starts_with(path) {
                continue;
            }
            if file.dirty {
                file.deleted = true;
            } else {
                self.discard(index);
            }
        }
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
    }

    fn discard(&mut self, index: usize) {
        self.files.remove(index);
        self.active = match self.active {
            _ if self.files.is_empty() => None,
            Some(active) if active >= index => {
                Some(active.saturating_sub(1).min(self.files.len() - 1))
            }
            active => active,
        };
    }
}
