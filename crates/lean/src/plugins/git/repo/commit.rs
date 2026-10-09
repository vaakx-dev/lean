use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use super::repository::{head_commit, open};
use super::staging::add_paths;

pub(crate) fn commit(root: &Path, message: &str, paths_to_stage: &[PathBuf]) -> Result<()> {
    let (mut repository, workdir) = open(root)?;
    let merge_heads = merge_heads(&mut repository)?;
    let mut index = repository.index()?;
    if !paths_to_stage.is_empty() {
        add_paths(&mut index, paths_to_stage, &workdir)?;
        index.write()?;
    }
    if index.has_conflicts() {
        bail!("resolve the merge conflicts before committing");
    }
    let tree = repository.find_tree(index.write_tree()?)?;
    let head = head_commit(&repository)?;
    if merge_heads.is_empty() && is_unchanged(&tree, head.as_ref()) {
        bail!("nothing to commit");
    }
    let signature = repository
        .signature()
        .context("set user.name and user.email in your git config")?;
    let mut parents: Vec<git2::Commit> = head.into_iter().collect();
    for merge_head in &merge_heads {
        parents.push(repository.find_commit(*merge_head)?);
    }
    let parents: Vec<&git2::Commit> = parents.iter().collect();
    let message = git2::message_prettify(message, None)?;
    repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &parents,
    )?;
    if !merge_heads.is_empty() {
        repository.cleanup_state()?;
    }
    Ok(())
}

fn is_unchanged(tree: &git2::Tree, head: Option<&git2::Commit>) -> bool {
    match head {
        Some(head) => head.tree_id() == tree.id(),
        None => tree.is_empty(),
    }
}

fn merge_heads(repository: &mut git2::Repository) -> Result<Vec<git2::Oid>> {
    let mut heads = Vec::new();
    if repository.state() == git2::RepositoryState::Merge {
        repository.mergehead_foreach(|oid| {
            heads.push(*oid);
            true
        })?;
    }
    Ok(heads)
}
