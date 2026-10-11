mod file_scan;
mod matcher;
mod path_filter;
mod preview;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::channel::mpsc::UnboundedSender;
use gpui_kit::SharedString;
use ignore::overrides::Override;
use ignore::{DirEntry, WalkBuilder};

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
    for path in files(&job.root, job.filter, &job.config) {
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

fn files(
    root: &Path,
    filter: Override,
    config: &SearchConfig,
) -> impl Iterator<Item = PathBuf> + use<> {
    let respect_gitignore = config.respect_gitignore;
    WalkBuilder::new(root)
        .hidden(!config.include_hidden)
        .parents(true)
        .git_ignore(respect_gitignore)
        .git_global(respect_gitignore)
        .git_exclude(respect_gitignore)
        .require_git(false)
        .follow_links(config.follow_symlinks)
        .overrides(filter)
        .max_filesize(Some(config.max_file_size))
        .sort_by_file_name(|left, right| left.cmp(right))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(DirEntry::into_path)
}
