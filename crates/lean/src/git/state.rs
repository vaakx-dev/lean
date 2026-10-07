use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::SharedString;

use super::FileStatus;
use super::history::{CommitSummary, read_history};
use super::repository::workdir;
use super::statuses::read_statuses;

pub(crate) struct GitState {
    pub(crate) workdir: PathBuf,
    pub(crate) branch: SharedString,
    pub(crate) statuses: HashMap<PathBuf, FileStatus>,
    pub(crate) staged: HashMap<PathBuf, FileStatus>,
    pub(crate) unstaged: HashMap<PathBuf, FileStatus>,
    pub(crate) changed_folders: HashSet<PathBuf>,
    pub(crate) history: Vec<CommitSummary>,
}

impl GitState {
    pub(crate) fn read(root: &Path) -> Result<Option<Self>> {
        let repository = match git2::Repository::discover(root) {
            Ok(repository) => repository,
            Err(error) if error.code() == git2::ErrorCode::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let Some(workdir) = workdir(&repository) else {
            return Ok(None);
        };
        let branch = branch_name(&repository)?;
        let statuses = read_statuses(&repository, &workdir)?;
        let changed_folders = changed_folders(&statuses.combined, &workdir);
        let history = read_history(&repository)?;
        Ok(Some(Self {
            workdir,
            branch: branch.into(),
            statuses: statuses.combined,
            staged: statuses.staged,
            unstaged: statuses.unstaged,
            changed_folders,
            history,
        }))
    }

    pub(crate) fn is_dirty(&self) -> bool {
        !self.statuses.is_empty()
    }
}

fn branch_name(repository: &git2::Repository) -> Result<String> {
    match repository.head() {
        Ok(head) if head.is_branch() => Ok(head.shorthand().unwrap_or("HEAD").to_string()),
        Ok(head) => Ok(head
            .target()
            .map(|oid| oid.to_string().chars().take(8).collect())
            .unwrap_or_else(|| "HEAD".to_string())),
        Err(error) if error.code() == git2::ErrorCode::UnbornBranch => {
            Ok(unborn_branch_name(repository).unwrap_or_else(|| "main".to_string()))
        }
        Err(error) => Err(error.into()),
    }
}

fn unborn_branch_name(repository: &git2::Repository) -> Option<String> {
    let head = repository.find_reference("HEAD").ok()?;
    let target = head.symbolic_target()?;
    Some(target.trim_start_matches("refs/heads/").to_string())
}

fn changed_folders(statuses: &HashMap<PathBuf, FileStatus>, workdir: &Path) -> HashSet<PathBuf> {
    let mut folders = HashSet::new();
    for path in statuses.keys() {
        let mut folder = path.parent();
        while let Some(current) = folder {
            if !current.starts_with(workdir) || !folders.insert(current.to_path_buf()) {
                break;
            }
            folder = current.parent();
        }
    }
    folders
}
