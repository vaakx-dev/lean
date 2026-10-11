use futures::future::Shared;
use gpui_kit::Task;

use super::disk_io::{DiskFile, DiskStamp};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SaveOutcome {
    Saved,
    Failed,
    ChangedOnDisk,
}

pub(super) type SaveInFlight = Shared<Task<SaveOutcome>>;

pub(super) struct DiskState {
    pub(super) saved_hash: u64,
    pub(super) stamp: Option<DiskStamp>,
    pub(super) written: Option<DiskStamp>,
    pub(super) conflict: bool,
    pub(super) saving: Option<SaveInFlight>,
    pub(super) changed_while_saving: bool,
    pub(super) _check: Task<()>,
    pub(super) _auto_save: Task<()>,
}

impl DiskState {
    pub(super) fn loaded(disk: &DiskFile) -> Self {
        Self {
            saved_hash: disk.hash,
            stamp: Some(disk.stamp),
            ..Self::detached()
        }
    }

    pub(super) fn detached() -> Self {
        Self {
            saved_hash: 0,
            stamp: None,
            written: None,
            conflict: false,
            saving: None,
            changed_while_saving: false,
            _check: Task::ready(()),
            _auto_save: Task::ready(()),
        }
    }

    pub(super) fn reloaded(&mut self, hash: u64, stamp: DiskStamp) {
        self.saved_hash = hash;
        self.stamp = Some(stamp);
        self.written = None;
        self.conflict = false;
    }

    pub(super) fn saved(&mut self, hash: u64, stamp: DiskStamp) {
        self.saved_hash = hash;
        self.stamp = Some(stamp);
        self.written = Some(stamp);
        self.conflict = false;
    }
}
