mod changes;
mod folders;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::{Context, Task};
use notify::Event;

use super::{Toolchain, Toolchains};
use crate::watcher::FolderWatch;

const REFRESH_DELAY: Duration = Duration::from_secs(1);

pub(super) fn watch(
    project_root: &Path,
    search_paths: &[PathBuf],
    available: &[Toolchain],
    cx: &mut Context<Toolchains>,
) -> Option<Task<()>> {
    let known: HashSet<PathBuf> = available
        .iter()
        .map(|toolchain| toolchain.prefix.clone())
        .collect();
    let is_environment_change = move |event: &Event| changes::is_relevant(event, &known);
    let mut environments = FolderWatch::new(REFRESH_DELAY, is_environment_change)
        .inspect_err(|error| log::warn!("failed to watch Python environments: {error}"))
        .ok()?;
    for folder in folders::watched_folders(project_root, search_paths, available) {
        if let Err(error) = environments.watch(&folder) {
            log::debug!("failed to watch {}: {error}", folder.display());
        }
    }
    Some(cx.spawn(async move |toolchains, cx| {
        while environments.next_change(cx).await {
            if toolchains
                .update(cx, |toolchains, cx| toolchains.refresh(cx))
                .is_err()
            {
                return;
            }
        }
    }))
}
