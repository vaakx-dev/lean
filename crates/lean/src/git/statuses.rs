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

pub(super) fn read_statuses(repository: &git2::Repository, workdir: &Path) -> Result<Statuses> {
    let mut options = git2::StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .renames_head_to_index(true);
    let entries = repository.statuses(Some(&mut options))?;
    let mut statuses = Statuses::default();
    for entry in entries.iter() {
        let status = entry.status();
        let entry_path = entry.path().map(PathBuf::from);
        let staged_path = delta_path(entry.head_to_index());
        let unstaged_path = delta_path(entry.index_to_workdir());
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
