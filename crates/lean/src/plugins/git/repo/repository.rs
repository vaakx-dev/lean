use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};

pub(crate) struct RepoFile {
    pub(crate) repository: git2::Repository,
    pub(crate) relative_path: PathBuf,
}

impl RepoFile {
    pub(crate) fn open(root: &Path, path: &Path) -> Result<Self> {
        let (repository, workdir) = open(root)?;
        let relative_path = workdir_relative(path, &workdir)?;
        Ok(Self {
            repository,
            relative_path,
        })
    }
}

pub(crate) fn workdir_relative(path: &Path, workdir: &Path) -> Result<PathBuf> {
    Ok(canonical(path)
        .strip_prefix(workdir)
        .with_context(|| format!("{} is outside the repository", path.display()))?
        .to_path_buf())
}

pub(crate) fn open(root: &Path) -> Result<(git2::Repository, PathBuf)> {
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
    if let Ok(path) = dunce::canonicalize(path) {
        return path;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => canonical(parent).join(name),
        _ => path.to_path_buf(),
    }
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

pub(crate) fn head_commit(repository: &git2::Repository) -> Result<Option<git2::Commit<'_>>> {
    match repository.head() {
        Ok(head) => Ok(Some(head.peel_to_commit()?)),
        Err(error)
            if matches!(
                error.code(),
                git2::ErrorCode::UnbornBranch | git2::ErrorCode::NotFound
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}
