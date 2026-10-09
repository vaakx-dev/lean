use std::path::{Path, PathBuf};

use crate::project::IgnoreCache;

#[derive(Default)]
pub(super) struct Walk {
    pub(super) folders: Vec<PathBuf>,
    pub(super) skipped: Vec<PathBuf>,
}

pub(super) fn walk(start: &Path, ignore: &IgnoreCache) -> Walk {
    let mut walk = Walk::default();
    let mut pending = vec![start.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let (ignored, kept) = ignore.split_ignored_folders(&folder, subfolders(&folder));
        pending.extend(kept);
        walk.skipped.extend(ignored);
        walk.folders.push(folder);
    }
    walk
}

fn subfolders(folder: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!("failed to read {}: {error}", folder.display());
            return Vec::new();
        }
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() != ".git")
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .map(|entry| entry.path())
        .collect()
}
