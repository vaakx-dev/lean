use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gpui_kit::{AppContext as _, Context, Task};

use super::{GitEvent, GitProject};
use crate::plugins::git::repo::{
    CommitHistory, GitReadError, GitState, ReadOptions, metadata_watch,
};
use crate::plugins::git::settings::{INCLUDE_UNTRACKED, SHOW_HISTORY, history_page_size};
use crate::watcher::{FsChanges, MetadataWatch};

const DEBOUNCE: Duration = Duration::from_millis(150);

type GitRead = Result<(Option<GitState>, MetadataWatch), GitReadError>;

pub(super) struct Refresh {
    running: bool,
    queued: bool,
    started: Option<Instant>,
    pub(super) views_stale: bool,
    pub(super) views_refreshed: Option<Instant>,
    scheduled: Option<Task<()>>,
    _task: Task<()>,
}

impl Default for Refresh {
    fn default() -> Self {
        Self {
            running: false,
            queued: false,
            started: None,
            views_stale: false,
            views_refreshed: None,
            scheduled: None,
            _task: Task::ready(()),
        }
    }
}

impl Refresh {
    pub(super) fn is_pending(&self) -> bool {
        self.running || self.scheduled.is_some()
    }
}

impl GitProject {
    pub(super) fn schedule_refresh(&mut self, cx: &mut Context<Self>) {
        if self.refresh.scheduled.is_some() {
            return;
        }
        self.refresh.scheduled = Some(cx.spawn(async move |git, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            git.update(cx, |git, cx| {
                git.refresh.scheduled = None;
                git.start_read(cx);
            })
            .ok();
        }));
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.refresh.views_stale = true;
        self.start_read(cx);
    }

    pub(super) fn start_read(&mut self, cx: &mut Context<Self>) {
        self.refresh.scheduled = None;
        if self.refresh.running {
            self.refresh.queued = true;
            return;
        }
        self.refresh.running = true;
        self.refresh._task = cx.spawn(async move |git, cx| {
            loop {
                let Ok((root, previous, options)) = git.update(cx, |git, cx| git.begin_read(cx))
                else {
                    break;
                };
                let result = cx
                    .background_spawn(async move { read_git(&root, previous, options) })
                    .await;
                if !git
                    .update(cx, |git, cx| git.finish_read(result, cx))
                    .unwrap_or(false)
                {
                    break;
                }
            }
        });
    }

    pub(crate) fn load_more_history(&mut self, cx: &mut Context<Self>) {
        self.history_limit += history_page_size(cx);
        self.start_read(cx);
    }

    pub(super) fn read_covers(&self, changes: &FsChanges) -> bool {
        changes
            .observed_at
            .zip(self.refresh.started)
            .is_some_and(|(observed, started)| started > observed)
    }

    fn begin_read(&mut self, cx: &Context<Self>) -> (PathBuf, Option<CommitHistory>, ReadOptions) {
        self.refresh.started = Some(Instant::now());
        let previous = self.state.as_ref().map(GitState::commit_history);
        let options = ReadOptions {
            include_untracked: INCLUDE_UNTRACKED.get(cx),
            history_limit: if SHOW_HISTORY.get(cx) {
                self.history_limit
            } else {
                0
            },
        };
        (self.root(cx), previous, options)
    }

    fn finish_read(&mut self, result: GitRead, cx: &mut Context<Self>) -> bool {
        let (state, error) = match result {
            Ok((state, watch)) => {
                self.project.read(cx).watch_metadata(watch);
                (state, None)
            }
            Err(error) => {
                log::warn!("failed to read git status: {}", error.message);
                (None, Some(error))
            }
        };
        let changed = self.state != state || self.error != error;
        if changed {
            self.state = state;
            self.error = error;
            self.publish_decorations(cx);
            cx.emit(GitEvent::Changed);
            cx.notify();
        }
        if std::mem::take(&mut self.refresh.views_stale) || changed {
            self.refresh_views(cx);
        }
        let queued = std::mem::take(&mut self.refresh.queued);
        self.refresh.running = queued;
        queued
    }
}

fn read_git(root: &Path, previous: Option<CommitHistory>, options: ReadOptions) -> GitRead {
    let state =
        GitState::read(root, previous, options).map_err(|error| GitReadError::new(&error, root))?;
    let git_dirs = state
        .as_ref()
        .map(|state| state.git_dirs.as_slice())
        .unwrap_or_default();
    let watch = metadata_watch(git_dirs);
    Ok((state, watch))
}
