use gpui_kit::Task;

use super::disk_io::{DiskFile, DiskStamp};

pub(super) struct DiskState {
    pub(super) saved_hash: u64,
    pub(super) stamp: Option<DiskStamp>,
    pub(super) conflict: bool,
    pub(super) saving: bool,
    pub(super) _check: Task<()>,
}

impl DiskState {
    pub(super) fn loaded(disk: &DiskFile) -> Self {
        Self {
            saved_hash: disk.hash,
            stamp: Some(disk.stamp),
            conflict: false,
            saving: false,
            _check: Task::ready(()),
        }
    }

    pub(super) fn detached() -> Self {
        Self {
            saved_hash: 0,
            stamp: None,
            conflict: false,
            saving: false,
            _check: Task::ready(()),
        }
    }

    pub(super) fn reloaded(&mut self, hash: u64, stamp: DiskStamp) {
        self.saved_hash = hash;
        self.stamp = Some(stamp);
        self.conflict = false;
    }
}
