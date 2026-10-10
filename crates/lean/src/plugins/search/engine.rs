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

use super::config::SearchConfig;
use super::query::SearchQuery;
use super::results::{FileResult, SearchBatch};
use matcher::Matcher;

const FLUSH_INTERVAL: Duration = Duration::from_millis(50);

pub(super) struct SearchJob {
    root: PathBuf,
    matcher: Matcher,
    filter: Override,
    config: SearchConfig,
}

pub(super) fn prepare(root: PathBuf, query: SearchQuery) -> Result<SearchJob, SharedString> {
    let matcher = Matcher::new(&query.pattern, query.options, query.config.smart_case)?;
    let filter = path_filter::build(&root, &query.include, &query.exclude, &query.config.exclude)?;
    Ok(SearchJob {
        root,
        matcher,
        filter,
        config: query.config,
    })
}

pub(super) fn run(job: SearchJob, sender: UnboundedSender<SearchBatch>) {
    let mut pending: Vec<FileResult> = Vec::new();
    let mut remaining = job.config.max_results;
    let mut truncated = false;
    let mut last_flush: Option<Instant> = None;
    for path in walk::files(&job.root, job.filter, &job.config) {
        if sender.is_closed() {
            return;
        }
        if remaining == 0 {
            if file_scan::contains_match(&path, &job.matcher) {
                truncated = true;
                break;
            }
            continue;
        }
        if let Some(scanned) = file_scan::scan(&job.root, path, &job.matcher, remaining) {
            remaining -= scanned.file.match_count;
            truncated = scanned.overflow;
            pending.push(scanned.file);
        }
        if truncated {
            break;
        }
        let due =
            remaining == 0 || last_flush.is_none_or(|flushed| flushed.elapsed() >= FLUSH_INTERVAL);
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
        truncated,
    };
    sender.unbounded_send(batch).ok();
}
