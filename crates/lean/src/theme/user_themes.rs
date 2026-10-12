use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, Global, Task};
use notify::{Event, EventKind};

use super::{appearance, catalog};
use crate::storage::config_file;
use crate::watcher::reload_on_change;

const FOLDER_NAME: &str = "themes";
const RELOAD_DELAY: Duration = Duration::from_millis(200);

static RELOADED: Mutex<Option<Vec<ThemeConfig>>> = Mutex::new(None);

struct UserThemesWatcher {
    _task: Task<()>,
}

impl Global for UserThemesWatcher {}

pub(super) fn load() -> Vec<ThemeConfig> {
    let reloaded = RELOADED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    reloaded.unwrap_or_else(|| themes_folder().map_or_else(Vec::new, |folder| read_folder(&folder)))
}

pub(super) fn watch(cx: &mut App) {
    let Some(folder) = themes_folder() else {
        return;
    };
    let is_theme_change = |event: &Event| {
        !matches!(event.kind, EventKind::Access(_))
            && event.paths.iter().any(|path| is_theme_file(path))
    };
    let watched = folder.clone();
    let read = move || read_folder(&watched);
    if let Some(task) = reload_on_change(&folder, RELOAD_DELAY, is_theme_change, read, apply, cx) {
        cx.set_global(UserThemesWatcher { _task: task });
    }
}

fn apply(themes: Vec<ThemeConfig>, cx: &mut App) {
    *RELOADED.lock().unwrap_or_else(PoisonError::into_inner) = Some(themes);
    catalog::load(cx);
    appearance::apply_current(cx);
}

fn themes_folder() -> Option<PathBuf> {
    config_file(FOLDER_NAME)
}

fn read_folder(folder: &Path) -> Vec<ThemeConfig> {
    let mut files = match std::fs::read_dir(folder) {
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
