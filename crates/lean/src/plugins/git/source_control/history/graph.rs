use crate::plugins::git::repo::CommitSummary;

pub(super) struct GraphRow {
    pub(super) lane: usize,
    pub(super) is_merge: bool,
    pub(super) lanes_above: Vec<bool>,
    pub(super) lanes_below: Vec<bool>,
    pub(super) merging_from: Vec<usize>,
    pub(super) branching_to: Vec<usize>,
}

impl GraphRow {
    pub(super) fn has_line_above(&self, lane: usize) -> bool {
        self.lanes_above.get(lane).copied().unwrap_or(false)
    }

    pub(super) fn has_line_below(&self, lane: usize) -> bool {
        self.lanes_below.get(lane).copied().unwrap_or(false)
    }

    pub(super) fn passing_lanes(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.lanes_above.len()).filter(move |&lane| {
            lane != self.lane
                && self.has_line_above(lane)
                && self.has_line_below(lane)
                && !self.merging_from.contains(&lane)
        })
    }
}

pub(super) fn build_graph(commits: &[CommitSummary]) -> (Vec<GraphRow>, usize) {
    let mut lanes: Vec<Option<git2::Oid>> = Vec::new();
    let mut rows = Vec::with_capacity(commits.len());
    let mut width = 1;
    for commit in commits {
        let lanes_above: Vec<bool> = lanes.iter().map(Option::is_some).collect();
        let expecting: Vec<usize> = lanes
            .iter()
            .enumerate()
            .filter(|(_, expected)| **expected == Some(commit.id))
            .map(|(lane, _)| lane)
            .collect();
        let lane = match expecting.first() {
            Some(&lane) => lane,
            None => free_lane(&mut lanes),
        };
        let merging_from: Vec<usize> = expecting.iter().skip(1).copied().collect();
        for &merged in &merging_from {
            lanes[merged] = None;
        }
        lanes[lane] = commit.parents.first().copied();
        let mut branching_to = Vec::new();
        for parent in commit.parents.iter().skip(1) {
            let target = match lanes.iter().position(|expected| *expected == Some(*parent)) {
                Some(existing) => existing,
                None => {
                    let opened = free_lane(&mut lanes);
                    lanes[opened] = Some(*parent);
                    opened
                }
            };
            branching_to.push(target);
        }
        while lanes.last() == Some(&None) {
            lanes.pop();
        }
        let lanes_below: Vec<bool> = lanes.iter().map(Option::is_some).collect();
        width = width
            .max(lanes_above.len())
            .max(lanes_below.len())
            .max(lane + 1);
        rows.push(GraphRow {
            lane,
            is_merge: commit.parents.len() > 1,
            lanes_above,
            lanes_below,
            merging_from,
            branching_to,
        });
    }
    (rows, width)
}

fn free_lane(lanes: &mut Vec<Option<git2::Oid>>) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(lane) => lane,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    }
}
