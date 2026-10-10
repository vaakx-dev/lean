use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui_kit::*;

use super::open_file::OpenFile;
use super::settings::PREVIEW_TABS;
use super::{EditorArea, EditorAreaEvent};

const REPEATED_OPEN: Duration = Duration::from_millis(500);

pub(super) struct LastOpen {
    path: PathBuf,
    at: Instant,
}

impl EditorArea {
    pub(super) fn observe_preview_tabs(cx: &mut Context<Self>) -> Subscription {
        PREVIEW_TABS.observe(cx, |area, enabled, cx| {
            if !enabled {
                area.pin_all(cx);
            }
        })
    }

    pub(super) fn opens_as_preview(&mut self, path: &Path, cx: &App) -> bool {
        let repeated = self
            .last_open
            .as_ref()
            .is_some_and(|last| last.path == path && last.at.elapsed() < REPEATED_OPEN);
        self.last_open = Some(LastOpen {
            path: path.to_path_buf(),
            at: Instant::now(),
        });
        PREVIEW_TABS.get(cx) && !repeated
    }

    pub(super) fn insert_preview(&mut self, mut file: OpenFile, cx: &mut Context<Self>) -> usize {
        file.preview = true;
        match self.files.iter().position(|open| open.preview) {
            Some(index) => {
                self.files[index] = file;
                cx.emit(EditorAreaEvent::OpenFilesChanged);
                index
            }
            None => self.insert(file, cx),
        }
    }

    pub(super) fn pin(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(file) = self.files.get_mut(index).filter(|file| file.preview) {
            file.preview = false;
            cx.notify();
        }
    }

    fn pin_all(&mut self, cx: &mut Context<Self>) {
        for file in &mut self.files {
            file.preview = false;
        }
        cx.notify();
    }
}
