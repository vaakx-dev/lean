use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver};
use gpui_kit::{App, AsyncApp};
use notify::event::{AccessKind, AccessMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use super::global::{Settings, replace_all};
use super::{load, save};

const RELOAD_DELAY: Duration = Duration::from_millis(150);

pub(super) fn start(file: PathBuf, cx: &mut App) {
    let Some(folder) = file.parent().map(Path::to_path_buf) else {
        return;
    };
    if let Err(error) = std::fs::create_dir_all(&folder) {
        log::warn!("failed to create {}: {error}", folder.display());
        return;
    }
    let Some(name) = file.file_name().map(OsString::from) else {
        return;
    };
    let (sender, receiver) = mpsc::unbounded::<()>();
    let handler = move |event: notify::Result<Event>| {
        if let Ok(event) = event
            && !is_noise(event.kind)
            && event
                .paths
                .iter()
                .any(|path| path.file_name() == Some(name.as_os_str()))
        {
            sender.unbounded_send(()).ok();
        }
    };
    let mut watcher = match RecommendedWatcher::new(handler, Config::default()) {
        Ok(watcher) => watcher,
        Err(error) => {
            log::warn!("failed to watch {}: {error}", file.display());
            return;
        }
    };
    if let Err(error) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
        log::warn!("failed to watch {}: {error}", folder.display());
        return;
    }
    let task = cx.spawn(async move |cx| reload_on_change(receiver, file, cx).await);
    cx.global_mut::<Settings>().watcher = Some((watcher, task));
}

async fn reload_on_change(mut changes: UnboundedReceiver<()>, file: PathBuf, cx: &mut AsyncApp) {
    while changes.next().await.is_some() {
        cx.background_executor().timer(RELOAD_DELAY).await;
        while changes.try_recv().is_ok() {}
        cx.update(|cx| reload(&file, cx));
    }
}

fn reload(file: &Path, cx: &mut App) {
    if save::is_busy(cx) {
        return;
    }
    match load::read(file) {
        Ok(values) => replace_all(values, cx),
        Err(error) => log::warn!("ignoring changes to {}: {error}", file.display()),
    }
}

fn is_noise(kind: EventKind) -> bool {
    matches!(kind, EventKind::Access(access) if access != AccessKind::Close(AccessMode::Write))
}
