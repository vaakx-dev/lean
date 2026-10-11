use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, Global, Task};
use notify::{Event, EventKind};

use super::{appearance, catalog};
use crate::storage::config_file;
use crate::watcher::FolderWatch;

const FOLDER_NAME: &str = "themes";
const RELOAD_DELAY: Duration = Duration::from_millis(200);

struct UserThemesWatcher {
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
    let is_theme_change = |event: &Event| {
        !matches!(event.kind, EventKind::Access(_))
            && event.paths.iter().any(|path| is_theme_file(path))
    };
    let mut changes = match FolderWatch::new(RELOAD_DELAY, is_theme_change) {
        Ok(changes) => changes,
        Err(error) => {
            log::warn!("failed to watch {}: {error}", folder.display());
            return;
        }
    };
    if let Err(error) = changes.watch(&folder) {
        log::warn!("failed to watch {}: {error}", folder.display());
        return;
    }
    let task = cx.spawn(async move |cx| {
        while changes.next_change(cx).await {
            cx.update(|cx| {
                catalog::load(cx);
                appearance::apply_current(cx);
            });
        }
    });
    cx.set_global(UserThemesWatcher { _task: task });
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
