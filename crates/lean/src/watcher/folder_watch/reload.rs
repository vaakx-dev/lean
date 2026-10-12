use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{App, AppContext as _, Task};
use notify::Event;

use super::FolderWatch;

pub(crate) fn reload_on_change<T: Send + 'static>(
    folder: &Path,
    delay: Duration,
    is_relevant: impl Fn(&Event) -> bool + Send + 'static,
    read: impl Fn() -> T + Send + Sync + 'static,
    apply: impl Fn(T, &mut App) + 'static,
    cx: &mut App,
) -> Option<Task<()>> {
    if let Err(error) = std::fs::create_dir_all(folder) {
        log::warn!("failed to create {}: {error}", folder.display());
        return None;
    }
    let mut changes = FolderWatch::new(delay, is_relevant)
        .and_then(|mut changes| changes.watch(folder).map(|()| changes))
        .inspect_err(|error| log::warn!("failed to watch {}: {error}", folder.display()))
        .ok()?;
    let read = Arc::new(read);
    Some(cx.spawn(async move |cx| {
        while changes.next_change(cx).await {
            let read = read.clone();
            let value = cx.background_spawn(async move { read() }).await;
            cx.update(|cx| apply(value, cx));
        }
    }))
}
