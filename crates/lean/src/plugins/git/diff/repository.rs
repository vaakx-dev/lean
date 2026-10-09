use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};

pub(super) struct RepoFile {
    pub(super) repository: git2::Repository,
    pub(super) relative_path: PathBuf,
}

pub(super) fn open_repo_file(repo_root: &Path, path: &Path) -> Result<RepoFile> {
    let repository = git2::Repository::discover(repo_root)?;
    let workdir = repository
        .workdir()
        .map(canonical)
        .ok_or_else(|| anyhow!("{} has no working directory", repo_root.display()))?;
    let relative_path = canonical(path)
        .strip_prefix(&workdir)
        .with_context(|| format!("{} is outside the repository", path.display()))?
        .to_path_buf();
    Ok(RepoFile {
        repository,
        relative_path,
    })
}

fn canonical(path: &Path) -> PathBuf {
    if let Ok(path) = dunce::canonicalize(path) {
        return path;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => canonical(parent).join(name),
        _ => path.to_path_buf(),
    }
}
