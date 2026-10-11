use std::fmt::Display;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::disk_io::{DiskFile, read_file};
use super::open_file::OpenFile;
use super::saving::BackupFile;
use super::{EditorArea, EditorAreaEvent};
use crate::files::file_name;
use crate::plugins::{FileViewer, viewer_for};

#[derive(Clone, Copy)]
pub(super) struct GoTo {
    pub(super) line: usize,
    pub(super) column: usize,
}

pub(super) enum Placement {
    Show { preview: bool, target: Option<GoTo> },
    Restore(BackupFile),
}

impl EditorArea {
    pub(crate) fn open(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.open_with(path, None, window, cx);
    }

    pub(crate) fn open_at(
        &mut self,
        path: PathBuf,
        line: usize,
        column: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_with(path, Some(GoTo { line, column }), window, cx);
    }

    pub(crate) fn open_pinned(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placement = Placement::Show {
            preview: false,
            target: None,
        };
        self.open_placed(path, placement, window, cx);
    }

    fn open_with(
        &mut self,
        path: PathBuf,
        target: Option<GoTo>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placement = Placement::Show {
            preview: self.opens_as_preview(&path, cx),
            target,
        };
        self.open_placed(path, placement, window, cx);
    }

    pub(super) fn open_placed(
        &mut self,
        path: PathBuf,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(index) = self.index_of(&path) {
            self.place(index, placement, window, cx);
            return;
        }
        let Some(viewer) = viewer_for(&path, cx) else {
            report_open_error(&path, &"no viewer can open it", &placement, window, cx);
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
            area.update_in(cx, |area, window, cx| match result {
                Ok(disk) => area.insert_file(path, viewer, disk, placement, window, cx),
                Err(error) => report_open_error(&path, &error, &placement, window, cx),
            })
            .ok();
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

    pub(super) fn go_to(
        &mut self,
        index: usize,
        target: GoTo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get(index) else {
            return;
        };
        file.view.go_to(target.line, target.column, window, cx);
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
            self.place(index, placement, window, cx);
            return;
        }
        let toolchains = self.toolchains.clone();
        let file = match OpenFile::open(path.clone(), viewer, disk, toolchains, window, cx) {
            Ok(file) => file,
            Err(error) => {
                report_open_error(&path, &format!("{error:#}"), &placement, window, cx);
                return;
            }
        };
        let index = match placement {
            Placement::Show { preview: true, .. } => self.insert_preview(file, cx),
            _ => self.insert(file, cx),
        };
        self.place(index, placement, window, cx);
    }

    pub(super) fn insert(&mut self, file: OpenFile, cx: &mut Context<Self>) -> usize {
        let index = self.active.map_or(self.files.len(), |active| active + 1);
        self.files.insert(index, file);
        cx.emit(EditorAreaEvent::OpenFilesChanged);
        index
    }

    fn place(
        &mut self,
        index: usize,
        placement: Placement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match placement {
            Placement::Show { preview, target } => {
                if !preview {
                    self.pin(index, cx);
                }
                self.activate(index, window, cx);
                if let Some(target) = target {
                    self.go_to(index, target, window, cx);
                }
            }
            Placement::Restore(backup) => {
                if self.active.is_none() {
                    self.activate(index, window, cx);
                }
                self.apply_backup(index, backup, window, cx);
            }
        }
    }
}

fn report_open_error(
    path: &Path,
    error: &dyn Display,
    placement: &Placement,
    window: &mut Window,
    cx: &mut App,
) {
    let name = file_name(path);
    match placement {
        Placement::Show { .. } => {
            log::warn!("failed to open {}: {error}", path.display());
            window.push_notification(format!("Can't open {name}: {error}"), cx);
        }
        Placement::Restore(_) => {
            log::warn!(
                "failed to restore unsaved changes to {}: {error}",
                path.display()
            );
            let message = format!("Couldn't restore unsaved changes to {name}: {error}");
            window.push_notification(Notification::warning(message).autohide(false), cx);
        }
    }
}
