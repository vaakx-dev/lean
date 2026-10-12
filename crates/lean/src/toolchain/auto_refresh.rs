mod changes;
mod folders;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::{AppContext as _, Context, Task};

use super::Toolchains;
use crate::watcher::FolderWatch;
use changes::{Changes, has_environment};

pub(super) use folders::watched_folders;

const REFRESH_DELAY: Duration = Duration::from_secs(1);

pub(super) struct AutoRefresh {
    folders: Vec<PathBuf>,
    changes: Changes,
    _task: Task<()>,
}

impl AutoRefresh {
    pub(super) fn start(
        folders: Vec<PathBuf>,
        known: HashSet<PathBuf>,
        cx: &mut Context<Toolchains>,
    ) -> Option<Self> {
        let changes = Changes::new(known);
        let recorder = changes.clone();
        let mut watch = FolderWatch::new(REFRESH_DELAY, move |event| recorder.record(event))
            .inspect_err(|error| log::warn!("failed to watch Python environments: {error}"))
            .ok()?;
        for folder in &folders {
            if let Err(error) = watch.watch(folder) {
                log::debug!("failed to watch {}: {error}", folder.display());
            }
        }
        let pending = changes.clone();
        let task = cx.spawn(async move |toolchains, cx| {
            while watch.next_change(cx).await {
                let (touched_known, created) = pending.take();
                let relevant = touched_known
                    || cx
                        .background_spawn(async move { has_environment(&created) })
                        .await;
                if relevant
                    && toolchains
                        .update(cx, |toolchains, cx| toolchains.refresh(cx))
                        .is_err()
                {
                    return;
                }
            }
        });
        Some(Self {
            folders,
            changes,
            _task: task,
        })
    }

    pub(super) fn watches(&self, folders: &[PathBuf]) -> bool {
        self.folders == folders
    }

    pub(super) fn set_known(&self, known: HashSet<PathBuf>) {
        self.changes.set_known(known);
    }
}
