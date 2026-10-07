use std::path::Path;

use anyhow::{Context as _, Result, bail};

use super::repository::{head_commit, open};
use super::staging::add_everything;

pub(crate) fn commit(root: &Path, message: &str, stage_everything: bool) -> Result<()> {
    let (mut repository, _) = open(root)?;
    let merge_heads = merge_heads(&mut repository)?;
    let mut index = repository.index()?;
    if stage_everything {
        add_everything(&mut index)?;
        index.write()?;
    }
    if index.has_conflicts() {
        bail!("resolve the merge conflicts before committing");
    }
    let tree = repository.find_tree(index.write_tree()?)?;
    let signature = repository
        .signature()
        .context("set user.name and user.email in your git config")?;
    let mut parents = Vec::new();
    if let Some(head) = head_commit(&repository)? {
        parents.push(head);
    }
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
