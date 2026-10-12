use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(in crate::explorer) struct FolderLoads {
    next_id: u64,
    pending: HashMap<PathBuf, PendingLoad>,
}

pub(super) struct PendingLoad {
    pub(super) reveal: Option<PathBuf>,
    id: u64,
}

impl FolderLoads {
    pub(super) fn begin(&mut self, folder: &Path, reveal: Option<PathBuf>) -> u64 {
        self.next_id += 1;
        let reveal = reveal.or_else(|| {
            self.pending
                .remove(folder)
                .and_then(|pending| pending.reveal)
        });
        let id = self.next_id;
        self.pending
            .insert(folder.to_path_buf(), PendingLoad { id, reveal });
        id
    }

    pub(super) fn finish(&mut self, folder: &Path, id: u64) -> Option<PendingLoad> {
        if self.pending.get(folder)?.id != id {
            return None;
        }
        self.pending.remove(folder)
    }

    pub(super) fn forget(&mut self, is_forgotten: impl Fn(&Path) -> bool) {
        self.pending.retain(|folder, _| !is_forgotten(folder));
    }

    pub(in crate::explorer) fn is_idle(&self) -> bool {
        self.pending.is_empty()
    }
}
