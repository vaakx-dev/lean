use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc::{self, UnboundedReceiver};
use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, AsyncApp, Global, Task};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use super::{appearance, catalog};
use crate::storage::config_file;

const FOLDER_NAME: &str = "themes";
const RELOAD_DELAY: Duration = Duration::from_millis(200);

struct UserThemesWatcher {
    _watcher: RecommendedWatcher,
    _task: Task<()>,
}

impl Global for UserThemesWatcher {}

pub(super) fn load() -> Vec<ThemeConfig> {
    let Some(folder) = themes_folder() else {
        return Vec::new();
    };
    let mut files = match std::fs::read_dir(&folder) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_theme_file(path))
            .collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };
    files.sort();
    files.iter().flat_map(|file| load_file(file)).collect()
}

pub(super) fn watch(cx: &mut App) {
    let Some(folder) = themes_folder() else {
        return;
    };
    if let Err(error) = std::fs::create_dir_all(&folder) {
        log::warn!("failed to create {}: {error}", folder.display());
        return;
    }
    let (sender, receiver) = mpsc::unbounded::<()>();
    let handler = move |event: notify::Result<Event>| {
        if let Ok(event) = event
            && !matches!(event.kind, EventKind::Access(_))
            && event.paths.iter().any(|path| is_theme_file(path))
        {
            sender.unbounded_send(()).ok();
        }
    };
    let mut watcher = match RecommendedWatcher::new(handler, Config::default()) {
        Ok(watcher) => watcher,
        Err(error) => {
            log::warn!("failed to watch {}: {error}", folder.display());
            return;
        }
    };
    if let Err(error) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
        log::warn!("failed to watch {}: {error}", folder.display());
        return;
    }
    let task = cx.spawn(async move |cx| reload_on_change(receiver, cx).await);
    cx.set_global(UserThemesWatcher {
        _watcher: watcher,
        _task: task,
    });
}

async fn reload_on_change(mut changes: UnboundedReceiver<()>, cx: &mut AsyncApp) {
    while changes.next().await.is_some() {
        cx.background_executor().timer(RELOAD_DELAY).await;
        while changes.try_recv().is_ok() {}
        cx.update(|cx| {
            catalog::load(cx);
            appearance::apply_current(cx);
        });
    }
}

fn themes_folder() -> Option<PathBuf> {
    config_file(FOLDER_NAME)
}

fn is_theme_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "json")
}

fn load_file(file: &Path) -> Vec<ThemeConfig> {
    let contents = match std::fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(error) => {
            log::warn!("failed to read the theme {}: {error}", file.display());
            return Vec::new();
        }
    };
    catalog::parse_theme_set(&contents).unwrap_or_else(|error| {
        log::warn!("failed to parse the theme {}: {error}", file.display());
        Vec::new()
    })
}
