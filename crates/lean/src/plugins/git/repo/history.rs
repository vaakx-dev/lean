mod order;

use std::collections::{BinaryHeap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::SharedString;

use super::repository::head_commit;
use order::children_before_parents;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CommitSummary {
    pub(crate) id: git2::Oid,
    pub(crate) short_id: SharedString,
    pub(crate) parents: Vec<git2::Oid>,
    pub(crate) summary: SharedString,
    pub(crate) message: SharedString,
    pub(crate) author: SharedString,
    pub(crate) time: i64,
}

#[derive(Clone, Default)]
pub(crate) struct CommitHistory {
    pub(super) head: Option<git2::Oid>,
    pub(super) limit: usize,
    pub(super) commits: Arc<[CommitSummary]>,
    pub(super) has_more: bool,
}

pub(super) fn read_history(
    repository: &git2::Repository,
    previous: Option<CommitHistory>,
    limit: usize,
) -> Result<CommitHistory> {
    if limit == 0 {
        return Ok(CommitHistory::default());
    }
    let Some(head) = head_commit(repository)? else {
        return Ok(CommitHistory::default());
    };
    if let Some(previous) =
        previous.filter(|previous| previous.head == Some(head.id()) && previous.limit == limit)
    {
        return Ok(previous);
    }
    let (commits, has_more) = walk_history(repository, &head, limit)?;
    Ok(CommitHistory {
        head: Some(head.id()),
        limit,
        commits: commits.into(),
        has_more,
    })
}

fn walk_history(
    repository: &git2::Repository,
    head: &git2::Commit,
    limit: usize,
) -> Result<(Vec<CommitSummary>, bool)> {
    let mut queue = BinaryHeap::from([(head.time().seconds(), head.id())]);
    let mut seen = HashSet::from([head.id()]);
    let mut history = Vec::with_capacity(limit);
    while history.len() < limit
        && let Some((_, id)) = queue.pop()
    {
        let commit = repository.find_commit(id)?;
        for parent in commit.parents() {
            if seen.insert(parent.id()) {
                queue.push((parent.time().seconds(), parent.id()));
            }
        }
        history.push(summarize(&commit));
    }
    Ok((children_before_parents(history), !queue.is_empty()))
}

fn summarize(commit: &git2::Commit) -> CommitSummary {
    let id = commit.id();
    let short_id: String = id.to_string().chars().take(7).collect();
    CommitSummary {
        id,
        short_id: short_id.into(),
        parents: commit.parent_ids().collect(),
        summary: commit.summary().unwrap_or_default().to_string().into(),
        message: String::from_utf8_lossy(commit.message_bytes())
            .trim_end()
            .to_string()
            .into(),
        author: commit
            .author()
            .name()
            .unwrap_or("Unknown")
            .to_string()
            .into(),
        time: commit.time().seconds(),
    }
}
