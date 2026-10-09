use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::CommitSummary;

pub(super) fn children_before_parents(commits: Vec<CommitSummary>) -> Vec<CommitSummary> {
    let order = topological_order(&commits);
    let mut slots: Vec<Option<CommitSummary>> = commits.into_iter().map(Some).collect();
    let mut ordered: Vec<CommitSummary> = order
        .into_iter()
        .filter_map(|index| slots.get_mut(index).and_then(Option::take))
        .collect();
    ordered.extend(slots.into_iter().flatten());
    ordered
}

fn topological_order(commits: &[CommitSummary]) -> Vec<usize> {
    let index_of: HashMap<git2::Oid, usize> = commits
        .iter()
        .enumerate()
        .map(|(index, commit)| (commit.id, index))
        .collect();
    let mut waiting_children = vec![0usize; commits.len()];
    for parent in commits.iter().flat_map(|commit| &commit.parents) {
        if let Some(&index) = index_of.get(parent) {
            waiting_children[index] += 1;
        }
    }
    let mut ready: BinaryHeap<Reverse<usize>> = waiting_children
        .iter()
        .enumerate()
        .filter(|(_, waiting)| **waiting == 0)
        .map(|(index, _)| Reverse(index))
        .collect();
    let mut order = Vec::with_capacity(commits.len());
    while let Some(Reverse(index)) = ready.pop() {
        order.push(index);
        let parents = commits.get(index).map(|commit| commit.parents.as_slice());
        for parent in parents.unwrap_or_default() {
            let Some(&parent_index) = index_of.get(parent) else {
                continue;
            };
            if let Some(waiting) = waiting_children.get_mut(parent_index) {
                *waiting = waiting.saturating_sub(1);
                if *waiting == 0 {
                    ready.push(Reverse(parent_index));
                }
            }
        }
    }
    order
}
