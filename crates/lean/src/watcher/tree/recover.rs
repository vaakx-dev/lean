use notify::RecursiveMode;

use super::WatchedTree;
use crate::watcher::filter::RawEvent;
use crate::watcher::{FsChanges, WATCH_EACH_FOLDER};

impl WatchedTree {
    pub(in crate::watcher) fn process(&mut self, mut events: Vec<RawEvent>) -> Option<FsChanges> {
        if events.iter().any(|event| event.retry) {
            self.recover();
            events.push(RawEvent::rescan());
        }
        self.classify(events)
    }

    fn recover(&mut self) {
        self.health.restore();
        if self.watcher.is_none() && !self.create_watcher() {
            return;
        }
        self.metadata_targets.clear();
        self.open_file_folders.clear();
        if !WATCH_EACH_FOLDER {
            let root = self.root.clone();
            self.unwatch(&root);
            if self.watch(&root, RecursiveMode::Recursive) {
                self.folders.insert(root);
            }
        }
    }
}
