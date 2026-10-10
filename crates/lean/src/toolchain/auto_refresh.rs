mod changes;
mod folders;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver};
use gpui_kit::{AsyncApp, Context, Task, WeakEntity};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as _};

use super::{Toolchain, Toolchains};

const REFRESH_DELAY: Duration = Duration::from_secs(1);

pub(super) struct AutoRefresh {
    _watcher: RecommendedWatcher,
    _task: Task<()>,
}

impl AutoRefresh {
    pub(super) fn start(
        project_root: &Path,
        search_paths: &[PathBuf],
        available: &[Toolchain],
        cx: &mut Context<Toolchains>,
    ) -> Option<Self> {
        let known: HashSet<PathBuf> = available
            .iter()
            .map(|toolchain| toolchain.prefix.clone())
            .collect();
        let (sender, receiver) = mpsc::unbounded::<()>();
        let handler = move |event: notify::Result<Event>| {
            if let Ok(event) = event
                && changes::is_relevant(&event, &known)
            {
                sender.unbounded_send(()).ok();
            }
        };
        let mut watcher = match RecommendedWatcher::new(handler, Config::default()) {
            Ok(watcher) => watcher,
            Err(error) => {
                log::warn!("failed to watch Python environments: {error}");
                return None;
            }
        };
        for folder in folders::watched_folders(project_root, search_paths, available) {
            if let Err(error) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
                log::debug!("failed to watch {}: {error}", folder.display());
            }
        }
        let task =
            cx.spawn(async move |toolchains, cx| refresh_on_change(receiver, toolchains, cx).await);
        Some(Self {
            _watcher: watcher,
            _task: task,
        })
    }
}

async fn refresh_on_change(
    mut changes: UnboundedReceiver<()>,
    toolchains: WeakEntity<Toolchains>,
    cx: &mut AsyncApp,
) {
    while changes.next().await.is_some() {
        cx.background_executor().timer(REFRESH_DELAY).await;
        while changes.try_recv().is_ok() {}
        if toolchains
            .update(cx, |toolchains, cx| toolchains.refresh(cx))
            .is_err()
        {
            return;
        }
    }
}
