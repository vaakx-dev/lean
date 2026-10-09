mod binary_file;
mod closing;
mod disk_io;
mod disk_state;
mod disk_sync;
mod file_actions;
pub(crate) mod file_text;
mod go_to;
mod minimap;
mod open_file;
mod opening;
mod opening_view;
mod path_changes;
mod saving;
mod tabs;
mod text_file;
mod watermark;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui_kit::component::v_flex;
use gpui_kit::*;

use crate::plugins::AnyFileView;
use crate::toolchain::Toolchains;
use open_file::OpenFile;
use watermark::render_watermark;

pub(crate) use text_file::TEXT_VIEWER;

pub(crate) enum EditorAreaEvent {
    ActiveEditorChanged,
    FileEdited(PathBuf),
    FileRecreated(PathBuf),
    OpenFilesChanged,
    Saved,
}

pub(crate) struct EditorArea {
    files: Vec<OpenFile>,
    active: Option<usize>,
    toolchains: Entity<Toolchains>,
    focus_handle: FocusHandle,
}

impl EventEmitter<EditorAreaEvent> for EditorArea {}

impl EditorArea {
    pub(crate) fn new(toolchains: Entity<Toolchains>, cx: &mut Context<Self>) -> Self {
        Self {
            files: Vec::new(),
            active: None,
            toolchains,
            focus_handle: cx.focus_handle(),
        }
    }

    pub(crate) fn open_paths(&self) -> HashSet<PathBuf> {
        self.open_file_views()
            .map(|(path, _)| path.to_path_buf())
            .collect()
    }

    pub(crate) fn open_file_views(&self) -> impl Iterator<Item = (&Path, &AnyFileView)> {
        self.files
            .iter()
            .filter(|file| file.is_file())
            .map(|file| (file.path.as_path(), &file.view))
    }

    pub(crate) fn unsaved_file_names(&self) -> Vec<SharedString> {
        self.files
            .iter()
            .filter(|file| file.dirty)
            .map(|file| file.name.clone())
            .collect()
    }

    pub(crate) fn running_work(&self, cx: &App) -> Vec<(&'static str, usize)> {
        let mut work: Vec<(&'static str, usize)> = Vec::new();
        for kind in self
            .files
            .iter()
            .filter_map(|file| file.view.running_work(cx))
        {
            match work.iter_mut().find(|(known, _)| *known == kind) {
                Some((_, count)) => *count += 1,
                None => work.push((kind, 1)),
            }
        }
        work
    }

    pub(crate) fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        self.active_file()
            .map(|file| file.view.status_items(cx))
            .unwrap_or_default()
    }

    pub(crate) fn focus_active(&self, window: &mut Window, cx: &mut App) {
        let focus_handle = match self.active_file() {
            Some(file) => file.view.focus_handle(cx),
            None => self.focus_handle.clone(),
        };
        window.focus(&focus_handle, cx);
    }

    fn active_file(&self) -> Option<&OpenFile> {
        self.files.get(self.active?)
    }

    fn index_of(&self, path: &Path) -> Option<usize> {
        self.files
            .iter()
            .position(|file| file.is_file() && file.path == path)
    }

    fn index_of_view(&self, key: &Path) -> Option<usize> {
        self.files
            .iter()
            .position(|file| !file.is_file() && file.path == key)
    }

    fn index_of_id(&self, id: u64) -> Option<usize> {
        self.files.iter().position(|file| file.id == id)
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
            Some(file) => area
                .child(self.render_tab_bar(cx))
                .child(div().flex_1().min_h_0().child(file.view.view())),
            None => area.child(render_watermark(cx)),
        }
    }
}
