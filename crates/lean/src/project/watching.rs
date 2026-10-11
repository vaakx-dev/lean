use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui_kit::Context;

use super::{FilesChanged, Project, ProjectEvent};
use crate::watcher::{FsChanges, MetadataWatch};

impl Project {
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
            self.rescan_now(cx);
        } else {
            let unwatched = self.watcher.unwatched_folders();
            if !unwatched.is_empty() {
                cx.emit(FilesChanged(Rc::new(FsChanges::unwatched(unwatched))));
            }
        }
        cx.emit(ProjectEvent::Activated);
    }

    fn rescan_now(&mut self, cx: &mut Context<Self>) {
        self.ignore.clear();
        self.apply_fs_changes(FsChanges::rescan(), cx);
    }
}
