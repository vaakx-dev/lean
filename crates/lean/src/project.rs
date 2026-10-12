mod directory;
mod exclusions;
mod ignore_cache;
mod ignore_rules;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Result;
use gpui_kit::{App, Context, EventEmitter, SharedString, Subscription, Task};

use crate::recent_projects::display_name;
use crate::watcher::{FsChanges, FsWatcher, MetadataWatch};

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
        let ignore = IgnoreCache::new(root.clone(), &EXCLUDE.get(cx));
        let watcher = FsWatcher::start(root.clone(), ignore.clone(), cx);
        let exclusions =
            EXCLUDE.observe(cx, |project, patterns, _| project.set_exclusions(&patterns));
        Self {
            name: display_name(&root).into(),
            root,
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

    pub(crate) fn read_directories(
        &self,
        folders: Vec<PathBuf>,
        cx: &App,
    ) -> Task<Vec<Result<Vec<DirectoryEntry>>>> {
        directory::read_directories(folders, self.ignore.clone(), cx)
    }

    pub(crate) fn changed_presence(
        &self,
        listed: Vec<(PathBuf, bool)>,
        cx: &App,
    ) -> Task<Vec<PathBuf>> {
        directory::changed_presence(listed, self.ignore.clone(), cx)
    }

    pub(crate) fn watches_folder(&self, folder: &Path) -> bool {
        self.watcher.watches(folder)
    }

    pub(crate) fn watch_open_files(&self, files: HashSet<PathBuf>) {
        self.watcher.set_open_files(files);
    }

    pub(crate) fn watch_metadata(&self, watch: MetadataWatch) {
        self.watcher.set_metadata(watch);
    }

    pub(crate) fn rescan(&mut self) {
        self.watcher.request_rescan();
    }

    pub(crate) fn refresh_after_activation(&mut self, cx: &mut Context<Self>) {
        if !self.watcher.is_healthy() {
            self.ignore.clear();
            self.apply_fs_changes(FsChanges::rescan(), cx);
        } else {
            let unwatched = self.watcher.unwatched_folders();
            if !unwatched.is_empty() {
                self.apply_fs_changes(FsChanges::unwatched(unwatched), cx);
            }
        }
        cx.emit(ProjectEvent::Activated);
    }

    pub(crate) fn apply_fs_changes(&mut self, changes: FsChanges, cx: &mut Context<Self>) {
        cx.emit(FilesChanged(Rc::new(changes)));
    }

    fn set_exclusions(&mut self, patterns: &[String]) {
        self.ignore.set_exclusions(patterns);
        self.watcher.request_rescan();
    }
}
