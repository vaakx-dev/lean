use std::ops::Range;

use super::preview::line_result;
use crate::plugins::search::results::LineResult;

pub(super) fn line_break(text: &str, from: usize) -> Range<usize> {
    let Some(offset) = text[from..].find('\n') else {
        return text.len()..text.len();
    };
    let newline = from + offset;
    let start = if text[..newline].ends_with('\r') {
        newline - 1
    } else {
        newline
    };
    start..newline + 1
}

pub(super) fn line_results(text: &str, matches: &[Range<usize>]) -> Vec<LineResult> {
    let mut results = Vec::new();
    let mut index = 0;
    let mut scanned = 0;
    let mut rest = matches;
    while let Some(first) = rest.first() {
        let gap = &text[scanned..first.start];
        index += gap.matches('\n').count();
        let start = gap
            .rfind('\n')
            .map_or(scanned, |newline| scanned + newline + 1);
        let end = line_break(text, first.start).start;
        let count = rest.iter().take_while(|found| found.start < end).count();
        let (line, next) = rest.split_at(count);
        let ranges = line
            .iter()
            .map(|found| found.start - start..found.end - start)
            .collect();
        results.push(line_result(index, &text[start..end], ranges));
        scanned = end;
        rest = next;
    }
    results
}
