use std::fs::File;
use std::io::Read as _;
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::Search;
use super::lines::line_results;
use crate::plugins::search::results::FileResult;

const BINARY_PROBE: u64 = 8 * 1024;

pub(super) fn scan(search: &Search, path: PathBuf) -> Option<FileResult> {
    let text = read_text(&path)?;
    let mut matches = search.matcher.find_iter(&text);
    let limit = search.budget.remaining();
    if limit == 0 {
        if matches.next().is_some() {
            search.budget.truncate();
        }
        return None;
    }
    let found: Vec<Range<usize>> = matches.take(limit.saturating_add(1)).collect();
    if found.is_empty() {
        return None;
    }
    let claimed = search.budget.claim(found.len());
    if claimed == 0 {
        return None;
    }
    let lines = line_results(&text, &found[..claimed]);
    Some(FileResult::new(&search.root, path, lines))
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
