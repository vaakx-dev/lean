mod directory;
mod exclusions;
mod ignore_cache;
mod ignore_rules;
mod watching;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
use gpui_kit::{App, Context, EventEmitter, SharedString, Subscription, Task};

use crate::watcher::{FsChanges, FsWatcher};

pub(crate) use directory::DirectoryEntry;
pub(crate) use exclusions::EXCLUDE;
pub(crate) use ignore_cache::IgnoreCache;

pub(crate) enum ProjectEvent {
    Activated,
    FilesTouched,
    RevealRequested(PathBuf),
    WatchFailed(SharedString),
}

pub(crate) struct FilesChanged(pub(crate) Rc<FsChanges>);

pub(crate) struct Project {
    root: PathBuf,
    name: SharedString,
    ignore: IgnoreCache,
    watcher: FsWatcher,
    _exclusions: Subscription,
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
        let ignore = IgnoreCache::new(root.clone(), &EXCLUDE.get(cx));
        let watcher = FsWatcher::start(root.clone(), ignore.clone(), cx);
        let exclusions =
            EXCLUDE.observe(cx, |project, patterns, _| project.set_exclusions(&patterns));
        Self {
            root,
            name,
            ignore,
            watcher,
            _exclusions: exclusions,
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

    pub(crate) fn is_excluded(&self, path: &Path) -> bool {
        self.ignore.is_excluded(path, path.is_dir())
    }

    pub(crate) fn apply_fs_changes(&mut self, changes: FsChanges, cx: &mut Context<Self>) {
        if !changes.is_empty() {
            cx.emit(FilesChanged(Rc::new(changes)));
        }
    }

    fn set_exclusions(&mut self, patterns: &[String]) {
        self.ignore.set_exclusions(patterns);
        self.watcher.request_rescan();
    }
}
