use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui_kit::{Context, SharedString};

use super::{FilesChanged, Project, ProjectEvent, WatchFailed};
use crate::watcher::{FsChanges, MetadataWatch};

impl Project {
    pub(crate) fn watches_folder(&self, folder: &Path) -> bool {
        self.watcher.watches(folder)
    }

    pub(crate) fn watch_open_files(&mut self, files: HashSet<PathBuf>) {
        if self.open_files == files {
            return;
        }
        self.open_files = files.clone();
        self.watcher.set_open_files(files);
    }

    pub(crate) fn watch_metadata(&mut self, watch: MetadataWatch) {
        if self.metadata.watches_same(&watch) {
            return;
        }
        self.metadata = watch.clone();
        self.watcher.set_metadata(watch);
    }

    pub(crate) fn reconcile_after_watch_start(&mut self, cx: &mut Context<Self>) {
        cx.emit(FilesChanged(Rc::new(FsChanges::reconcile())));
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

    pub(crate) fn report_watch_failure(&mut self, message: SharedString, cx: &mut Context<Self>) {
        cx.emit(WatchFailed(message));
    }

    fn rescan_now(&mut self, cx: &mut Context<Self>) {
        self.ignore.clear();
        self.apply_fs_changes(FsChanges::rescan(), cx);
    }
}
