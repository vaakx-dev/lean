use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiskEntry {
    Missing,
    File,
    Folder,
}

impl DiskEntry {
    pub(crate) fn listed(is_dir: bool) -> Self {
        if is_dir { Self::Folder } else { Self::File }
    }

    pub(super) fn of(path: &Path) -> Self {
        match path.symlink_metadata() {
            Err(_) => Self::Missing,
            Ok(_) => Self::listed(path.is_dir()),
        }
    }
}
