use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::FileStatus;

#[derive(Default)]
pub(super) struct Statuses {
    pub(super) combined: HashMap<PathBuf, FileStatus>,
    pub(super) staged: HashMap<PathBuf, FileStatus>,
    pub(super) unstaged: HashMap<PathBuf, FileStatus>,
}

pub(super) fn read_statuses(
    repository: &git2::Repository,
    workdir: &Path,
    scope: Option<&Path>,
    include_untracked: bool,
) -> Result<Statuses> {
    let mut options = git2::StatusOptions::new();
    options
        .include_untracked(include_untracked)
        .recurse_untracked_dirs(include_untracked)
        .include_ignored(false)
        .renames_head_to_index(true);
    if let Some(scope) = scope {
        options.pathspec(scope).disable_pathspec_match(true);
    }
    let in_scope = |path: &PathBuf| scope.is_none_or(|scope| path.starts_with(scope));
    let entries = repository.statuses(Some(&mut options))?;
    let mut statuses = Statuses::default();
    for entry in entries.iter() {
        let status = entry.status();
        let entry_path = entry.path().map(PathBuf::from).filter(in_scope);
        let staged_path = delta_path(entry.head_to_index()).filter(in_scope);
        let unstaged_path = delta_path(entry.index_to_workdir()).filter(in_scope);
        let newest_path = unstaged_path
            .clone()
            .or_else(|| staged_path.clone())
            .or_else(|| entry_path.clone());
        if let Some(file_status) = FileStatus::from_index(status)
            && let Some(path) = staged_path.or_else(|| entry_path.clone())
        {
            statuses.staged.insert(workdir.join(path), file_status);
        }
        if let Some(file_status) = FileStatus::from_worktree(status)
            && let Some(path) = unstaged_path.or_else(|| entry_path.clone())
        {
            statuses.unstaged.insert(workdir.join(path), file_status);
        }
        if let Some(file_status) = FileStatus::from_git(status)
            && let Some(path) = newest_path
        {
            statuses.combined.insert(workdir.join(path), file_status);
        }
    }
    Ok(statuses)
}

fn delta_path(delta: Option<git2::DiffDelta<'_>>) -> Option<PathBuf> {
    delta?.new_file().path().map(Path::to_path_buf)
}
