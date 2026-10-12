use std::mem;
use std::time::{Duration, Instant};

use ignore::{DirEntry, Error, ParallelVisitor, ParallelVisitorBuilder, WalkState};

use super::{Search, file_scan};
use crate::plugins::search::results::{FileResult, SearchBatch};

const FLUSH_INTERVAL: Duration = Duration::from_millis(50);

struct Worker<'s> {
    search: &'s Search,
    pending: Vec<FileResult>,
    last_flush: Option<Instant>,
}

impl<'s> ParallelVisitorBuilder<'s> for &'s Search {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(Worker {
            search: *self,
            pending: Vec::new(),
            last_flush: None,
        })
    }
}

impl ParallelVisitor for Worker<'_> {
    fn visit(&mut self, entry: Result<DirEntry, Error>) -> WalkState {
        if self.search.sender.is_closed() {
            return WalkState::Quit;
        }
        let Some(path) = entry.ok().filter(is_file).map(DirEntry::into_path) else {
            return WalkState::Continue;
        };
        if let Some(file) = file_scan::scan(self.search, path) {
            self.pending.push(file);
            if self
                .last_flush
                .is_none_or(|flushed| flushed.elapsed() >= FLUSH_INTERVAL)
            {
                self.flush();
            }
        }
        if self.search.budget.is_truncated() {
            WalkState::Quit
        } else {
            WalkState::Continue
        }
    }
}

impl Worker<'_> {
    fn flush(&mut self) {
        let batch = SearchBatch {
            files: mem::take(&mut self.pending),
            truncated: false,
        };
        self.search.sender.unbounded_send(batch).ok();
        self.last_flush = Some(Instant::now());
    }
}

impl Drop for Worker<'_> {
    fn drop(&mut self) {
        if !self.pending.is_empty() {
            self.flush();
        }
    }
}

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|kind| kind.is_file())
}
