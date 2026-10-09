mod file_scan;
mod matcher;
mod path_filter;
mod preview;
mod walk;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;
use gpui_kit::SharedString;
use ignore::overrides::Override;

use super::query::SearchQuery;
use super::results::{FileResult, SearchBatch};
use matcher::Matcher;

pub(super) const MAX_MATCHES: usize = 2000;
const FLUSH_INTERVAL: Duration = Duration::from_millis(50);

pub(super) struct SearchJob {
    root: PathBuf,
    matcher: Matcher,
    filter: Override,
}

pub(super) fn prepare(root: PathBuf, query: &SearchQuery) -> Result<SearchJob, SharedString> {
    let matcher = Matcher::new(&query.pattern, query.options)?;
    let filter = path_filter::build(&root, &query.include, &query.exclude)?;
    Ok(SearchJob {
        root,
        matcher,
        filter,
    })
}

pub(super) fn run(job: SearchJob, sender: UnboundedSender<SearchBatch>) {
    let mut pending: Vec<FileResult> = Vec::new();
    let mut remaining = MAX_MATCHES;
    let mut last_flush: Option<Instant> = None;
    for path in walk::files(&job.root, job.filter) {
        if sender.is_closed() {
            return;
        }
        if let Some(file) = file_scan::scan(&job.root, path, &job.matcher, remaining) {
            remaining = remaining.saturating_sub(file.match_count);
            pending.push(file);
        }
        if remaining == 0 {
            break;
        }
        let due = last_flush.is_none_or(|flushed| flushed.elapsed() >= FLUSH_INTERVAL);
        if !pending.is_empty() && due {
            let batch = SearchBatch {
                files: std::mem::take(&mut pending),
                truncated: false,
            };
            if sender.unbounded_send(batch).is_err() {
                return;
            }
            last_flush = Some(Instant::now());
        }
    }
    let batch = SearchBatch {
        files: pending,
        truncated: remaining == 0,
    };
    sender.unbounded_send(batch).ok();
}
