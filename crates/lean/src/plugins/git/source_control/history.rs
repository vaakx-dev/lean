mod commit_row;
mod graph;
mod graph_paint;
mod relative_time;

use std::rc::Rc;

use crate::plugins::git::repo::CommitSummary;
use graph::{GraphRow, build_graph};

pub(super) struct HistoryEntry {
    commit: CommitSummary,
    graph: Rc<GraphRow>,
}

#[derive(Default)]
pub(super) struct History {
    pub(super) entries: Vec<HistoryEntry>,
    lanes: usize,
}

impl History {
    pub(super) fn build(commits: &[CommitSummary]) -> Self {
        let (graph_rows, lanes) = build_graph(commits);
        let entries = commits
            .iter()
            .cloned()
            .zip(graph_rows)
            .map(|(commit, graph)| HistoryEntry {
                commit,
                graph: Rc::new(graph),
            })
            .collect();
        Self { entries, lanes }
    }
}
