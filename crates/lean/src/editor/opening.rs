use std::path::PathBuf;

use anyhow::Context as _;
use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::open_file::OpenFile;
use super::{EditorArea, EditorAreaEvent};

impl EditorArea {
    pub(crate) fn open(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.index_of(&path) {
            self.activate(index, window, cx);
            return;
        }
        let read = cx.background_spawn({
            let path = path.clone();
            async move {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("{} is not a UTF-8 text file", path.display()))
            }
        });
        cx.spawn_in(window, async move |area, cx| {
            let result = read.await;
            let update = area.update_in(cx, |area, window, cx| match result {
                Ok(content) => area.insert_file(path, content, window, cx),
                Err(error) => {
                    log::warn!("failed to open {}: {error:#}", path.display());
                    window.push_notification(format!("Can't open file: {error:#}"), cx);
                }
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while opening a file: {error:#}");
            }
        })
        .detach();
    }

    pub(crate) fn activate_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active
            && !self.files.is_empty()
        {
            self.activate((active + 1) % self.files.len(), window, cx);
        }
    }

    pub(crate) fn activate_previous(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active
            && !self.files.is_empty()
        {
            let count = self.files.len();
            self.activate((active + count - 1) % count, window, cx);
        }
    }

    pub(super) fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(file) = self.files.get(index) else {
            return;
        };
        let focus_handle = file.editor.focus_handle(cx);
        self.active = Some(index);
        window.focus(&focus_handle, cx);
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
    }

    fn insert_file(
        &mut self,
        path: PathBuf,
        content: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of(&path) {
            self.activate(index, window, cx);
            return;
        }
        let file = OpenFile::new(path, content, window, cx);
        let index = self.active.map_or(self.files.len(), |active| active + 1);
        self.files.insert(index, file);
        self.activate(index, window, cx);
    }
}
