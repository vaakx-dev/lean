mod budget;
mod file_scan;
mod lines;
mod matcher;
mod path_filter;
mod preview;
mod worker;

use std::path::{Path, PathBuf};

use futures::channel::mpsc::UnboundedSender;
use gpui_kit::SharedString;
use ignore::overrides::Override;
use ignore::{WalkBuilder, WalkParallel};

use super::config::SearchConfig;
use super::query::SearchQuery;
use super::results::SearchBatch;
use budget::Budget;
use matcher::Matcher;

pub(super) struct SearchJob {
    root: PathBuf,
    matcher: Matcher,
    filter: Override,
    config: SearchConfig,
}

struct Search {
    root: PathBuf,
    matcher: Matcher,
    budget: Budget,
    sender: UnboundedSender<SearchBatch>,
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
    let walker = walker(&job.root, job.filter, &job.config);
    let search = Search {
        root: job.root,
        matcher: job.matcher,
        budget: Budget::new(job.config.max_results),
        sender,
    };
    walker.visit(&mut &search);
    let batch = SearchBatch {
        files: Vec::new(),
        truncated: search.budget.is_truncated(),
    };
    search.sender.unbounded_send(batch).ok();
}

fn walker(root: &Path, filter: Override, config: &SearchConfig) -> WalkParallel {
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
        .build_parallel()
}
