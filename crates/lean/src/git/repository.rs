use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};

pub(super) fn open(root: &Path) -> Result<(git2::Repository, PathBuf)> {
    let repository = git2::Repository::discover(root)?;
    let workdir = workdir(&repository)
        .ok_or_else(|| anyhow!("{} has no working directory", root.display()))?;
    Ok((repository, workdir))
}

pub(super) fn workdir(repository: &git2::Repository) -> Option<PathBuf> {
    let workdir = repository.workdir()?;
    Some(dunce::canonicalize(workdir).unwrap_or_else(|error| {
        log::debug!("failed to canonicalize {}: {error}", workdir.display());
        workdir.to_path_buf()
    }))
}

pub(super) fn relative_paths<'a>(paths: &'a [PathBuf], workdir: &Path) -> Result<Vec<&'a Path>> {
    paths
        .iter()
        .map(|path| {
            path.strip_prefix(workdir)
                .with_context(|| format!("{} is outside the repository", path.display()))
        })
        .collect()
}

pub(super) fn head_commit(repository: &git2::Repository) -> Result<Option<git2::Commit<'_>>> {
    match repository.head() {
        Ok(head) => Ok(Some(head.peel_to_commit()?)),
        Err(error) if error.code() == git2::ErrorCode::UnbornBranch => Ok(None),
        Err(error) => Err(error.into()),
    }
}
