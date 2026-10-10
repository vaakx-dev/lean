use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use gpui_kit::*;

use super::disk_io::DiskFile;
use super::disk_state::DiskState;
use super::{EditorArea, EditorAreaEvent};
use crate::files::file_name;
use crate::plugins::{AnyFileView, FileViewEvent, FileViewer, OpenRequest};
use crate::toolchain::Toolchains;

static NEXT_FILE_ID: AtomicU64 = AtomicU64::new(0);

pub(super) enum TabKind {
    File,
    View { icon: SharedString },
}

pub(super) struct OpenFile {
    pub(super) id: u64,
    pub(super) path: PathBuf,
    pub(super) name: SharedString,
    pub(super) kind: TabKind,
    pub(super) dirty: bool,
    pub(super) deleted: bool,
    pub(super) preview: bool,
    pub(super) version: u64,
    pub(super) disk: DiskState,
    pub(super) view: AnyFileView,
    _subscription: Subscription,
}

impl OpenFile {
    pub(super) fn open(
        path: PathBuf,
        viewer: FileViewer,
        disk: DiskFile,
        toolchains: Entity<Toolchains>,
        window: &mut Window,
        cx: &mut Context<EditorArea>,
    ) -> Result<Self> {
        let state = DiskState::loaded(&disk);
        let request = OpenRequest {
            path: path.clone(),
            bytes: disk.bytes,
            toolchains,
        };
        let view = (viewer.open)(request, window, cx)?;
        let name = file_name(&path);
        Ok(Self::new(path, name, TabKind::File, state, view, cx))
    }

    pub(super) fn detached(
        key: PathBuf,
        title: SharedString,
        icon: SharedString,
        view: AnyFileView,
        cx: &mut Context<EditorArea>,
    ) -> Self {
        let kind = TabKind::View { icon };
        Self::new(key, title, kind, DiskState::detached(), view, cx)
    }

    fn new(
        path: PathBuf,
        name: SharedString,
        kind: TabKind,
        disk: DiskState,
        view: AnyFileView,
        cx: &mut Context<EditorArea>,
    ) -> Self {
        let id = NEXT_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let area = cx.weak_entity();
        let subscription = view.subscribe(
            cx,
            Box::new(move |event: &FileViewEvent, cx: &mut App| {
                let update = area.update(cx, |area, cx| area.handle_view_event(id, event, cx));
                if let Err(error) = update {
                    log::debug!("editor area dropped while a file view changed: {error:#}");
                }
            }),
        );
        Self {
            id,
            path,
            name,
            kind,
            dirty: false,
            deleted: false,
            preview: false,
            version: 0,
            disk,
            view,
            _subscription: subscription,
        }
    }

    pub(super) fn is_file(&self) -> bool {
        matches!(self.kind, TabKind::File)
    }

    pub(super) fn set_path(&mut self, path: PathBuf, cx: &mut App) {
        self.view.set_path(&path, cx);
        self.name = file_name(&path);
        self.path = path;
        self.deleted = false;
    }
}

impl EditorArea {
    fn handle_view_event(&mut self, id: u64, event: &FileViewEvent, cx: &mut Context<Self>) {
        let Some(index) = self.index_of_id(id) else {
            return;
        };
        match event {
            FileViewEvent::Edited => {
                let file = &mut self.files[index];
                file.version += 1;
                if !file.is_file() {
                    return;
                }
                let dirty = file.view.differs_from_saved(cx);
                if file.dirty != dirty || (dirty && file.preview) {
                    file.dirty = dirty;
                    file.preview &= !dirty;
                    cx.notify();
                }
                cx.emit(EditorAreaEvent::FileEdited(file.path.clone()));
            }
            FileViewEvent::StatusChanged => cx.emit(EditorAreaEvent::ActiveEditorChanged),
        }
    }
}
