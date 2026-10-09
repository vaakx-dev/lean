use std::ops::Range;

use crate::plugins::search::results::LineResult;

const CONTEXT_BEFORE: usize = 40;
const MAX_LEAD: usize = 60;
const MAX_PREVIEW: usize = 300;
const ELLIPSIS: &str = "…";

pub(super) fn line_result(index: usize, line: &str, ranges: Vec<Range<usize>>) -> LineResult {
    let first = ranges.first().map_or(0, |range| range.start);
    let column = line[..first].chars().count();
    let indent = line.len() - line.trim_start().len();
    let mut start = indent.min(first);
    let mut prefix = "";
    if first - start > MAX_LEAD {
        start = floor_char_boundary(line, first - CONTEXT_BEFORE);
        prefix = ELLIPSIS;
    }
    let end = floor_char_boundary(line, (start + MAX_PREVIEW).min(line.len()));
    let shift = |offset: usize| offset - start + prefix.len();
    let highlights = ranges
        .iter()
        .filter(|range| range.start < end)
        .map(|range| shift(range.start)..shift(range.end.min(end)))
        .collect();
    LineResult {
        line: index,
        column,
        match_count: ranges.len(),
        preview: format!("{prefix}{}", &line[start..end]).into(),
        highlights,
    }
}

fn floor_char_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
