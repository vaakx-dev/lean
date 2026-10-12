mod reload;
mod shared_watcher;

use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver};
use gpui_kit::AsyncApp;
use notify::Event;

pub(crate) use reload::reload_on_change;

pub(crate) struct FolderWatch {
    id: u64,
    folders: Vec<PathBuf>,
    changes: UnboundedReceiver<()>,
    delay: Duration,
}

impl FolderWatch {
    pub(crate) fn new(
        delay: Duration,
        is_relevant: impl Fn(&Event) -> bool + Send + 'static,
    ) -> notify::Result<Self> {
        let (sender, changes) = mpsc::unbounded();
        let id = shared_watcher::subscribe(Box::new(is_relevant), sender)?;
        Ok(Self {
            id,
            folders: Vec::new(),
            changes,
            delay,
        })
    }

    pub(crate) fn watch(&mut self, folder: &Path) -> notify::Result<()> {
        shared_watcher::watch(self.id, folder)?;
        self.folders.push(folder.to_path_buf());
        Ok(())
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

impl Drop for FolderWatch {
    fn drop(&mut self) {
        shared_watcher::unsubscribe(self.id, &self.folders);
    }
}
