use std::fs::File;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use super::matcher::Matcher;
use super::preview::line_result;
use crate::plugins::search::results::FileResult;

const BINARY_PROBE: u64 = 8 * 1024;

pub(super) struct ScannedFile {
    pub(super) file: FileResult,
    pub(super) overflow: bool,
}

pub(super) fn scan(
    root: &Path,
    path: PathBuf,
    matcher: &Matcher,
    budget: usize,
) -> Option<ScannedFile> {
    let text = read_text(&path)?;
    if !matcher.is_match(&text) {
        return None;
    }
    let mut remaining = budget;
    let mut overflow = false;
    let mut lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let mut ranges = matcher.find_in_line(line);
        if ranges.is_empty() {
            continue;
        }
        if ranges.len() > remaining {
            overflow = true;
            ranges.truncate(remaining);
        }
        if ranges.is_empty() {
            break;
        }
        remaining -= ranges.len();
        lines.push(line_result(index, line, ranges));
        if overflow {
            break;
        }
    }
    if lines.is_empty() {
        return None;
    }
    Some(ScannedFile {
        file: FileResult::new(root, path, lines),
        overflow,
    })
}

pub(super) fn contains_match(path: &Path, matcher: &Matcher) -> bool {
    read_text(path).is_some_and(|text| {
        matcher.is_match(&text) && text.lines().any(|line| matcher.has_match_in_line(line))
    })
}

fn read_text(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(BINARY_PROBE)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.contains(&0) {
        return None;
    }
    file.read_to_end(&mut bytes).ok()?;
    Some(match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned(),
    })
}
