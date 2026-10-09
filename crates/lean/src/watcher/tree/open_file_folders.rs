use std::collections::HashSet;
use std::path::{Path, PathBuf};

use notify::RecursiveMode;

use super::WatchedTree;
use crate::watcher::WATCH_EACH_FOLDER;

impl WatchedTree {
    pub(super) fn watch_open_file_folders(&mut self) {
        if !WATCH_EACH_FOLDER {
            return;
        }
        let wanted = self.skipped_open_file_folders();
        let stale: Vec<PathBuf> = self
            .open_file_folders
            .difference(&wanted)
            .filter(|folder| !self.folders.contains(folder))
            .cloned()
            .collect();
        for folder in &stale {
            self.unwatch(folder);
        }
        let mut watched = HashSet::new();
        for folder in wanted {
            if self.open_file_folders.contains(&folder)
                || self.watch(&folder, RecursiveMode::NonRecursive)
            {
                watched.insert(folder);
            }
        }
        self.open_file_folders = watched;
    }

    fn skipped_open_file_folders(&self) -> HashSet<PathBuf> {
        self.open_files
            .read()
            .iter()
            .filter(|file| self.skipped.contains_parent_of(file))
            .filter_map(|file| file.parent())
            .filter(|folder| !self.folders.contains(folder))
            .map(Path::to_path_buf)
            .collect()
    }
}
