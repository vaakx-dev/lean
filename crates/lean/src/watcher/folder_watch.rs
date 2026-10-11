use std::path::Path;
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver};
use gpui_kit::AsyncApp;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as _};

pub(crate) struct FolderWatch {
    watcher: RecommendedWatcher,
    changes: UnboundedReceiver<()>,
    delay: Duration,
}

impl FolderWatch {
    pub(crate) fn new(
        delay: Duration,
        is_relevant: impl Fn(&Event) -> bool + Send + 'static,
    ) -> notify::Result<Self> {
        let (sender, changes) = mpsc::unbounded();
        let watcher = RecommendedWatcher::new(
            move |event: notify::Result<Event>| {
                if event.is_ok_and(|event| is_relevant(&event)) {
                    sender.unbounded_send(()).ok();
                }
            },
            Config::default(),
        )?;
        Ok(Self {
            watcher,
            changes,
            delay,
        })
    }

    pub(crate) fn watch(&mut self, folder: &Path) -> notify::Result<()> {
        self.watcher.watch(folder, RecursiveMode::NonRecursive)
    }

    pub(crate) async fn next_change(&mut self, cx: &AsyncApp) -> bool {
        if self.changes.next().await.is_none() {
            return false;
        }
        cx.background_executor().timer(self.delay).await;
        while self.changes.try_recv().is_ok() {}
        true
    }
}
