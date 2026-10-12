mod commit_row;
mod graph;
mod graph_paint;

use std::rc::Rc;
use std::sync::Arc;

use crate::plugins::git::repo::CommitSummary;
use graph::{GraphRow, build_graph};

#[derive(Default)]
pub(super) struct History {
    commits: Arc<[CommitSummary]>,
    graph: Vec<Rc<GraphRow>>,
    lanes: usize,
}

impl History {
    pub(super) fn update(&mut self, commits: &Arc<[CommitSummary]>) {
        if Arc::ptr_eq(&self.commits, commits) {
            return;
        }
        let (graph, lanes) = build_graph(commits);
        *self = Self {
            commits: Arc::clone(commits),
            graph: graph.into_iter().map(Rc::new).collect(),
            lanes,
        };
    }

    pub(super) fn len(&self) -> usize {
        self.commits.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }

    fn entry(&self, index: usize) -> Option<(&CommitSummary, &Rc<GraphRow>)> {
        Some((self.commits.get(index)?, self.graph.get(index)?))
    }
}
