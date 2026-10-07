mod directory;

use std::path::{Path, PathBuf};

use gpui_kit::{AppContext as _, Context, EventEmitter, SharedString, Task};

use crate::git::{FileStatus, GitReadError, GitState};

pub(crate) use directory::{DirectoryEntry, read_directory};

pub(crate) enum ProjectEvent {
    GitChanged,
}

pub(crate) struct Project {
    root: PathBuf,
    name: SharedString,
    git: Option<GitState>,
    git_error: Option<GitReadError>,
    _refresh_task: Task<()>,
}

impl EventEmitter<ProjectEvent> for Project {}

impl Project {
    pub(crate) fn new(root: PathBuf, cx: &mut Context<Self>) -> Self {
        let name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string_lossy().into_owned())
            .into();
        let mut project = Self {
            root,
            name,
            git: None,
            git_error: None,
            _refresh_task: Task::ready(()),
        };
        project.refresh_git(cx);
        project
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn name(&self) -> &SharedString {
        &self.name
    }

    pub(crate) fn git(&self) -> Option<&GitState> {
        self.git.as_ref()
    }

    pub(crate) fn git_error(&self) -> Option<&GitReadError> {
        self.git_error.as_ref()
    }

    pub(crate) fn status_for(&self, path: &Path) -> Option<FileStatus> {
        self.git.as_ref()?.statuses.get(path).copied()
    }

    pub(crate) fn folder_has_changes(&self, path: &Path) -> bool {
        self.git
            .as_ref()
            .is_some_and(|git| git.changed_folders.contains(path))
    }

    pub(crate) fn refresh_git(&mut self, cx: &mut Context<Self>) {
        let root = self.root.clone();
        let task = cx.background_spawn(async move {
            GitState::read(&root).map_err(|error| GitReadError::new(&error, &root))
        });
        self._refresh_task = cx.spawn(async move |project, cx| {
            let (git, git_error) = match task.await {
                Ok(git) => (git, None),
                Err(error) => {
                    log::warn!("failed to read git status: {}", error.message);
                    (None, Some(error))
                }
            };
            if let Err(error) = project.update(cx, |project, cx| {
                project.git = git;
                project.git_error = git_error;
                cx.emit(ProjectEvent::GitChanged);
                cx.notify();
            }) {
                log::debug!("project dropped before git status finished: {error:#}");
            }
        });
    }
}
