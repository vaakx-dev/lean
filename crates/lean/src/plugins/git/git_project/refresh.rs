use std::path::PathBuf;
use std::time::Instant;

use gpui_kit::{AppContext as _, Context, Task};

use super::gutter::GutterScope;
use super::{GitEvent, GitProject};
use crate::plugins::git::repo::{CommitHistory, GitReadError, GitState, metadata_watch};
use crate::watcher::FsChanges;

type GitRead = Result<Option<GitState>, GitReadError>;

pub(super) struct Refresh {
    running: bool,
    queued: bool,
    started: Option<Instant>,
    _task: Task<()>,
}

impl Default for Refresh {
    fn default() -> Self {
        Self {
            running: false,
            queued: false,
            started: None,
            _task: Task::ready(()),
        }
    }
}

impl GitProject {
    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.refresh.running {
            self.refresh.queued = true;
            return;
        }
        self.refresh.running = true;
        self.refresh._task = cx.spawn(async move |git, cx| {
            loop {
                let Ok((root, previous)) = git.update(cx, |git, cx| git.begin_read(cx)) else {
                    break;
                };
                let result = cx
                    .background_spawn(async move {
                        GitState::read(&root, previous)
                            .map_err(|error| GitReadError::new(&error, &root))
                    })
                    .await;
                match git.update(cx, |git, cx| git.finish_read(result, cx)) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(error) => {
                        log::debug!("git state dropped before the status read finished: {error:#}");
                        break;
                    }
                }
            }
        });
    }

    pub(super) fn files_changed(&mut self, changes: &FsChanges, cx: &mut Context<Self>) {
        let affects_git = changes.metadata || changes.touches_files();
        if affects_git && !self.read_covers(changes) {
            self.refresh(cx);
        }
    }

    fn read_covers(&self, changes: &FsChanges) -> bool {
        !changes.rescan
            && changes
                .observed_at
                .zip(self.refresh.started)
                .is_some_and(|(observed, started)| started > observed)
    }

    fn begin_read(&mut self, cx: &Context<Self>) -> (PathBuf, Option<CommitHistory>) {
        self.refresh.started = Some(Instant::now());
        let previous = self.state.as_ref().map(GitState::commit_history);
        (self.root(cx), previous)
    }

    fn finish_read(&mut self, result: GitRead, cx: &mut Context<Self>) -> bool {
        let read_succeeded = result.is_ok();
        let (state, error) = match result {
            Ok(state) => (state, None),
            Err(error) => {
                log::warn!("failed to read git status: {}", error.message);
                (None, Some(error))
            }
        };
        if self.state != state || self.error != error {
            self.state = state;
            self.error = error;
            self.publish_decorations(cx);
            cx.emit(GitEvent::Changed);
            cx.notify();
        }
        if read_succeeded {
            self.watch_git_dirs(cx);
        }
        self.refresh_diff_views(cx);
        self.schedule_gutter(GutterScope::All, cx);
        let queued = std::mem::take(&mut self.refresh.queued);
        self.refresh.running = queued;
        queued
    }

    fn watch_git_dirs(&self, cx: &mut Context<Self>) {
        let git_dirs = self
            .state
            .as_ref()
            .map(|state| state.git_dirs.as_slice())
            .unwrap_or_default();
        let watch = metadata_watch(git_dirs);
        self.project
            .update(cx, |project, _| project.watch_metadata(watch));
    }
}
