mod closing;
mod minimap;
mod open_file;
mod opening;
mod path_changes;
mod saving;
mod tabs;
mod watermark;

use std::path::{Path, PathBuf};

use gpui_kit::component::v_flex;
use gpui_kit::*;

use open_file::OpenFile;
use watermark::render_watermark;

pub(crate) enum EditorAreaEvent {
    ActiveEditorChanged,
    FileRecreated(PathBuf),
}

pub(crate) struct EditorArea {
    files: Vec<OpenFile>,
    active: Option<usize>,
    focus_handle: FocusHandle,
}

impl EventEmitter<EditorAreaEvent> for EditorArea {}

impl EditorArea {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            files: Vec::new(),
            active: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub(crate) fn active_file(&self) -> Option<&OpenFile> {
        self.files.get(self.active?)
    }

    pub(crate) fn has_unsaved_files(&self) -> bool {
        self.files.iter().any(|file| file.dirty)
    }

    pub(crate) fn focus_active(&self, window: &mut Window, cx: &mut App) {
        let focus_handle = match self.active_file() {
            Some(file) => file.editor.focus_handle(cx),
            None => self.focus_handle.clone(),
        };
        window.focus(&focus_handle, cx);
    }

    fn index_of(&self, path: &Path) -> Option<usize> {
        self.files.iter().position(|file| file.path == path)
    }
}

impl Focusable for EditorArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorArea {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let area = v_flex()
            .id("editor-area")
            .track_focus(&self.focus_handle)
            .size_full();
        match self.active_file() {
            Some(file) => area.child(self.render_tabs(cx)).child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .child(file.render_editor(cx)),
                    )
                    .child(file.minimap.clone()),
            ),
            None => area.child(render_watermark(cx)),
        }
    }
}
