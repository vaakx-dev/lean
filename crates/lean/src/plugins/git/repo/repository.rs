use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};

pub(super) fn open(root: &Path) -> Result<(git2::Repository, PathBuf)> {
    let repository = git2::Repository::discover(root)?;
    let workdir = workdir(&repository)
        .ok_or_else(|| anyhow!("{} has no working directory", root.display()))?;
    Ok((repository, workdir))
}

pub(super) fn workdir(repository: &git2::Repository) -> Option<PathBuf> {
    repository.workdir().map(canonical)
}

pub(super) fn git_dirs(repository: &git2::Repository) -> Vec<PathBuf> {
    let mut dirs = vec![canonical(repository.path())];
    let common = canonical(repository.commondir());
    if !dirs.contains(&common) {
        dirs.push(common);
    }
    dirs
}

pub(super) fn canonical(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|error| {
        log::debug!("failed to canonicalize {}: {error}", path.display());
        path.to_path_buf()
    })
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
