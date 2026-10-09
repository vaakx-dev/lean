mod directory;
mod ignore_cache;
mod ignore_rules;
mod watching;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
use gpui_kit::{App, Context, EventEmitter, SharedString, Task};

use crate::watcher::{FsChanges, FsWatcher, MetadataWatch};

pub(crate) use directory::DirectoryEntry;
pub(crate) use ignore_cache::IgnoreCache;

pub(crate) enum ProjectEvent {
    Activated,
    FilesTouched,
}

pub(crate) struct FilesChanged(pub(crate) Rc<FsChanges>);

pub(crate) struct Project {
    root: PathBuf,
    name: SharedString,
    ignore: IgnoreCache,
    metadata: MetadataWatch,
    watcher: FsWatcher,
    open_files: HashSet<PathBuf>,
}

impl EventEmitter<ProjectEvent> for Project {}
impl EventEmitter<FilesChanged> for Project {}

impl Project {
    pub(crate) fn new(root: PathBuf, cx: &mut Context<Self>) -> Self {
        let name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string_lossy().into_owned())
            .into();
        let ignore = IgnoreCache::new(root.clone());
        let metadata = MetadataWatch::default();
        let watcher = FsWatcher::start(
            root.clone(),
            metadata.clone(),
            ignore.clone(),
            HashSet::new(),
            cx,
        );
        Self {
            root,
            name,
            ignore,
            metadata,
            watcher,
            open_files: HashSet::new(),
        }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn name(&self) -> &SharedString {
        &self.name
    }

    pub(crate) fn read_directory(
        &self,
        path: &Path,
        cx: &App,
    ) -> Task<Result<Vec<DirectoryEntry>>> {
        directory::read_directory(path, self.ignore.clone(), cx)
    }

    pub(crate) fn files_touched(&mut self, cx: &mut Context<Self>) {
        cx.emit(ProjectEvent::FilesTouched);
    }

    pub(crate) fn apply_fs_changes(&mut self, changes: FsChanges, cx: &mut Context<Self>) {
        if !changes.is_empty() {
            cx.emit(FilesChanged(Rc::new(changes)));
        }
    }
}
