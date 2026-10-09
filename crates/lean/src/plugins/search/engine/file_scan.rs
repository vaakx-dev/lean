use std::fs;
use std::path::{Path, PathBuf};

use super::matcher::Matcher;
use super::preview::line_result;
use crate::plugins::search::results::FileResult;

const BINARY_PROBE: usize = 8 * 1024;

pub(super) fn scan(
    root: &Path,
    path: PathBuf,
    matcher: &Matcher,
    budget: usize,
) -> Option<FileResult> {
    let bytes = fs::read(&path).ok()?;
    if is_binary(&bytes) {
        return None;
    }
    let text = String::from_utf8_lossy(&bytes);
    if !matcher.is_match(&text) {
        return None;
    }
    let mut remaining = budget;
    let mut lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if remaining == 0 {
            break;
        }
        let mut ranges = matcher.find_in_line(line);
        if ranges.is_empty() {
            continue;
        }
        ranges.truncate(remaining);
        remaining -= ranges.len();
        lines.push(line_result(index, line, ranges));
    }
    if lines.is_empty() {
        return None;
    }
    Some(FileResult::new(root, path, lines))
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(BINARY_PROBE)].contains(&0)
}
