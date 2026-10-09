use std::path::{Path, PathBuf};

use ignore::overrides::Override;
use ignore::{DirEntry, WalkBuilder};

const MAX_FILE_SIZE: u64 = 4 * 1024 * 1024;

pub(super) fn files(root: &Path, filter: Override) -> impl Iterator<Item = PathBuf> + use<> {
    WalkBuilder::new(root)
        .hidden(true)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .overrides(filter)
        .max_filesize(Some(MAX_FILE_SIZE))
        .sort_by_file_name(|left, right| left.cmp(right))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(DirEntry::into_path)
}
