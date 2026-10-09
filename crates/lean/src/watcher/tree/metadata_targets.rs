use std::path::PathBuf;

use notify::RecursiveMode;

use super::WatchedTree;
use crate::watcher::WATCH_EACH_FOLDER;

impl WatchedTree {
    pub(super) fn watch_metadata_targets(&mut self) {
        let wanted = self.metadata.targets(&self.root, WATCH_EACH_FOLDER);
        if wanted == self.metadata_targets {
            return;
        }
        let stale: Vec<PathBuf> = self
            .metadata_targets
            .iter()
            .filter(|target| !wanted.contains(target))
            .map(|(path, _)| path.clone())
            .filter(|path| !self.folders.contains(path))
            .collect();
        for path in &stale {
            self.unwatch(path);
        }
        let added: Vec<(PathBuf, RecursiveMode)> = wanted
            .iter()
            .filter(|target| !self.metadata_targets.contains(target))
            .cloned()
            .collect();
        for (path, mode) in added {
            self.watch(&path, mode);
        }
        self.metadata_targets = wanted;
    }
}
