use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use super::repository::{head_commit, open, relative_paths};

pub(crate) fn stage(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let (repository, workdir) = open(root)?;
    let mut index = repository.index()?;
    add_paths(&mut index, paths, &workdir)?;
    index.write()?;
    Ok(())
}

pub(super) fn add_paths(index: &mut git2::Index, paths: &[PathBuf], workdir: &Path) -> Result<()> {
    for (path, relative_path) in paths.iter().zip(relative_paths(paths, workdir)?) {
        if path.exists() || path.is_symlink() {
            index.add_path(relative_path)?;
        } else {
            index.remove_path(relative_path)?;
        }
    }
    Ok(())
}

pub(crate) fn unstage(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let (repository, workdir) = open(root)?;
    let head = head_commit(&repository)?;
    let relative_paths = relative_paths(paths, &workdir)?;
    repository.reset_default(
        head.as_ref().map(|commit| commit.as_object()),
        relative_paths,
    )?;
    Ok(())
}

pub(crate) fn discard(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let (repository, workdir) = open(root)?;
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.force().disable_pathspec_match(true);
    let mut has_tracked_paths = false;
    for (path, relative_path) in paths.iter().zip(relative_paths(paths, &workdir)?) {
        if repository.status_file(relative_path)?.is_wt_new() {
            delete_untracked(path)?;
        } else {
            checkout.path(relative_path);
            has_tracked_paths = true;
        }
    }
    if has_tracked_paths {
        repository.checkout_index(None, Some(&mut checkout))?;
    }
    Ok(())
}

fn delete_untracked(path: &Path) -> Result<()> {
    let result = if path.is_dir() && !path.is_symlink() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.with_context(|| format!("failed to delete {}", path.display()))
}
