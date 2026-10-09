use std::path::{Path, PathBuf};

use super::WATCH_EACH_FOLDER;
use super::shared_paths::SharedPaths;
use super::skipped::SkippedFolders;

pub(super) fn covers(watched: &SharedPaths, skipped: &SkippedFolders, folder: &Path) -> bool {
    if WATCH_EACH_FOLDER {
        return watched.contains(folder);
    }
    let Some(root) = recursive_root(watched, folder) else {
        return false;
    };
    !skipped.contains_self_or_parent_of(folder) && !crosses_symlink(&root, folder)
}

fn recursive_root(watched: &SharedPaths, folder: &Path) -> Option<PathBuf> {
    watched
        .read()
        .iter()
        .find(|root| folder.starts_with(root))
        .cloned()
}

fn crosses_symlink(root: &Path, folder: &Path) -> bool {
    folder
        .ancestors()
        .take_while(|ancestor| *ancestor != root)
        .any(|ancestor| {
            ancestor
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
}
