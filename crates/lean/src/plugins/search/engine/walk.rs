use std::path::{Path, PathBuf};

use ignore::overrides::Override;
use ignore::{DirEntry, WalkBuilder};

use crate::plugins::search::config::SearchConfig;

pub(super) fn files(
    root: &Path,
    filter: Override,
    config: &SearchConfig,
) -> impl Iterator<Item = PathBuf> + use<> {
    let respect_gitignore = config.respect_gitignore;
    WalkBuilder::new(root)
        .hidden(!config.include_hidden)
        .parents(true)
        .git_ignore(respect_gitignore)
        .git_global(respect_gitignore)
        .git_exclude(respect_gitignore)
        .require_git(false)
        .follow_links(config.follow_symlinks)
        .overrides(filter)
        .max_filesize(Some(config.max_file_size))
        .sort_by_file_name(|left, right| left.cmp(right))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(DirEntry::into_path)
}
