mod metadata_targets;
mod open_file_folders;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use futures::channel::mpsc::UnboundedSender;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher as _};

use super::classify::classify;
use super::filter::{EventFilter, RawEvent};
use super::health::Health;
use super::metadata::SharedMetadata;
use super::shared_paths::SharedPaths;
use super::skipped::SkippedFolders;
use super::walk::walk;
use super::{FsChanges, WATCH_EACH_FOLDER};
use crate::project::IgnoreCache;

pub(super) struct WatchedTree {
    root: PathBuf,
    ignore: IgnoreCache,
    skipped: SkippedFolders,
    health: Health,
    folders: SharedPaths,
    open_files: SharedPaths,
    open_file_folders: HashSet<PathBuf>,
    metadata: SharedMetadata,
    metadata_targets: Vec<(PathBuf, RecursiveMode)>,
    watcher: Option<RecommendedWatcher>,
}

impl WatchedTree {
    pub(super) fn new(
        root: PathBuf,
        ignore: IgnoreCache,
        health: Health,
        open_files: SharedPaths,
        metadata: SharedMetadata,
    ) -> Self {
        Self {
            skipped: SkippedFolders::new(&root),
            root,
            ignore,
            health,
            folders: SharedPaths::default(),
            open_files,
            open_file_folders: HashSet::new(),
            metadata,
            metadata_targets: Vec::new(),
            watcher: None,
        }
    }

    pub(super) fn skipped(&self) -> &SkippedFolders {
        &self.skipped
    }

    pub(super) fn watched(&self) -> &SharedPaths {
        &self.folders
    }

    pub(super) fn start(mut self, filter: EventFilter, sender: UnboundedSender<RawEvent>) -> Self {
        let health = self.health.clone();
        let handler = move |event: notify::Result<notify::Event>| match event {
            Ok(event) => {
                if let Some(raw) = filter.filter(event) {
                    sender.unbounded_send(raw).ok();
                }
            }
            Err(error) => health.report(&error),
        };
        let config = Config::default().with_follow_symlinks(false);
        match RecommendedWatcher::new(handler, config) {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(error) => {
                log::warn!("failed to start the file watcher: {error}");
                self.health.fail();
                return self;
            }
        }
        let root = self.root.clone();
        if WATCH_EACH_FOLDER {
            self.scan(&root);
        } else if self.watch(&root, RecursiveMode::Recursive) {
            self.folders.insert(root);
        }
        self.watch_metadata_targets();
        self.watch_open_file_folders();
        self
    }

    pub(super) fn classify(&mut self, events: Vec<RawEvent>) -> Option<FsChanges> {
        let changes = classify(events, &self.root, &self.ignore, &self.skipped);
        if let Some(changes) = &changes {
            if changes.rescan {
                self.rescan();
            } else if WATCH_EACH_FOLDER {
                self.follow_new_folders(changes);
            }
        }
        self.watch_metadata_targets();
        self.watch_open_file_folders();
        changes
    }

    fn follow_new_folders(&mut self, changes: &FsChanges) {
        let folders: Vec<PathBuf> = changes
            .paths
            .iter()
            .chain(&changes.ignored)
            .filter(|path| is_real_folder(path))
            .cloned()
            .collect();
        for folder in folders {
            self.scan(&folder);
        }
    }

    fn scan(&mut self, folder: &Path) {
        if self.ignore.is_ignored(folder) {
            self.skipped.insert(folder.to_path_buf());
            return;
        }
        let walk = walk(folder, &self.ignore);
        for folder in walk.folders {
            if self.watch(&folder, RecursiveMode::NonRecursive) {
                self.folders.insert(folder);
            }
        }
        self.skipped.extend(walk.skipped);
    }

    fn rescan(&mut self) {
        if !WATCH_EACH_FOLDER {
            self.skipped.replace(HashSet::new());
            return;
        }
        let walk = walk(&self.root, &self.ignore);
        let wanted: HashSet<PathBuf> = walk.folders.into_iter().collect();
        let stale: Vec<PathBuf> = self
            .folders
            .read()
            .difference(&wanted)
            .filter(|folder| !self.open_file_folders.contains(*folder))
            .cloned()
            .collect();
        for folder in &stale {
            self.unwatch(folder);
        }
        let watched = wanted
            .into_iter()
            .filter(|folder| self.watch(folder, RecursiveMode::NonRecursive))
            .collect();
        self.folders.replace(watched);
        self.skipped.replace(walk.skipped.into_iter().collect());
    }

    fn watch(&mut self, path: &Path, mode: RecursiveMode) -> bool {
        let Some(watcher) = self.watcher.as_mut().filter(|_| self.health.is_ok()) else {
            return false;
        };
        match watcher.watch(path, mode) {
            Ok(()) => true,
            Err(error) => {
                self.health.report(&error);
                false
            }
        }
    }

    fn unwatch(&mut self, path: &Path) {
        let Some(watcher) = self.watcher.as_mut() else {
            return;
        };
        if let Err(error) = watcher.unwatch(path) {
            log::debug!("failed to unwatch {}: {error}", path.display());
        }
    }
}

fn is_real_folder(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|metadata| metadata.is_dir())
}
