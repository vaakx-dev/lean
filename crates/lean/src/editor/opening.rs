use std::fmt::Display;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::disk_io::{DiskFile, read_file};
use super::go_to::GoTo;
use super::open_file::OpenFile;
use super::{EditorArea, EditorAreaEvent};
use crate::files::file_name;
use crate::plugins::{FileViewer, viewer_for};

impl EditorArea {
    pub(crate) fn open(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.open_with(path, None, window, cx);
    }

    pub(crate) fn open_pinned(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placement = Placement {
            preview: false,
            target: None,
        };
        self.open_placed(path, placement, window, cx);
    }

    pub(super) fn open_with(
        &mut self,
        path: PathBuf,
        target: Option<GoTo>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placement = Placement {
            preview: self.opens_as_preview(&path, cx),
            target,
        };
        self.open_placed(path, placement, window, cx);
    }

    fn open_placed(
        &mut self,
        path: PathBuf,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of(&path) {
            self.show(index, placement, window, cx);
            return;
        }
        let Some(viewer) = viewer_for(&path, cx) else {
            report_open_error(&path, &"no viewer can open it", window, cx);
            return;
        };
        let read = cx.background_spawn({
            let path = path.clone();
            async move {
                read_file(&path).inspect(|disk| {
                    if let Some(prepare) = viewer.prepare {
                        prepare(&path, &disk.bytes);
                    }
                })
            }
        });
        cx.spawn_in(window, async move |area, cx| {
            let result = read.await;
            let update = area.update_in(cx, |area, window, cx| match result {
                Ok(disk) => area.insert_file(path, viewer, disk, placement, window, cx),
                Err(error) => report_open_error(&path, &error, window, cx),
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
        let focus_handle = file.view.focus_handle(cx);
        self.active = Some(index);
        window.focus(&focus_handle, cx);
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
        cx.notify();
    }

    fn insert_file(
        &mut self,
        path: PathBuf,
        viewer: FileViewer,
        disk: DiskFile,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of(&path) {
            self.show(index, placement, window, cx);
            return;
        }
        let toolchains = self.toolchains.clone();
        let file = match OpenFile::open(path.clone(), viewer, disk, toolchains, window, cx) {
            Ok(file) => file,
            Err(error) => {
                report_open_error(&path, &format!("{error:#}"), window, cx);
                return;
            }
        };
        let index = if placement.preview {
            self.insert_preview(file, cx)
        } else {
            self.insert(file, cx)
        };
        self.show(index, placement, window, cx);
    }

    pub(super) fn insert(&mut self, file: OpenFile, cx: &mut Context<Self>) -> usize {
        let index = self.active.map_or(self.files.len(), |active| active + 1);
        self.files.insert(index, file);
        cx.emit(EditorAreaEvent::OpenFilesChanged);
        index
    }

    fn show(
        &mut self,
        index: usize,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !placement.preview {
            self.pin(index, cx);
        }
        self.activate(index, window, cx);
        if let Some(target) = placement.target {
            self.go_to(index, target, window, cx);
        }
    }
}

#[derive(Clone, Copy)]
struct Placement {
    preview: bool,
    target: Option<GoTo>,
}

fn report_open_error(path: &Path, error: &dyn Display, window: &mut Window, cx: &mut App) {
    log::warn!("failed to open {}: {error}", path.display());
    window.push_notification(format!("Can't open {}: {error}", file_name(path)), cx);
}
