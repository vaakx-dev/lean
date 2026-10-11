mod menu;
mod picker;
pub(crate) mod settings;

use std::path::{Path, PathBuf};

use gpui_kit::{App, Global};

use crate::storage::{QueuedWriter, read_state, state_file};
use settings::RECENT_LIMIT;

pub(crate) use menu::open_recent_menu;
pub(crate) use picker::open_picker;

const FILE_NAME: &str = "recent-projects.json";

static WRITER: QueuedWriter = QueuedWriter::new();

struct RecentProjects {
    current: Option<PathBuf>,
    paths: Vec<PathBuf>,
}

impl Global for RecentProjects {}

pub(crate) fn init(cx: &mut App) {
    let mut paths: Vec<PathBuf> = read_state(FILE_NAME).unwrap_or_default();
    let count = paths.len();
    paths.retain(|path| path.is_dir());
    if paths.len() != count {
        save(paths.clone(), cx);
    }
    cx.set_global(RecentProjects {
        current: None,
        paths,
    });
}

pub(crate) fn last_opened(cx: &App) -> Option<PathBuf> {
    cx.global::<RecentProjects>()
        .paths
        .iter()
        .find(|path| path.is_dir())
        .cloned()
}

pub(crate) fn others(cx: &App) -> Vec<PathBuf> {
    let recent = cx.global::<RecentProjects>();
    recent
        .paths
        .iter()
        .take(RECENT_LIMIT.get(cx))
        .filter(|path| recent.current.as_ref() != Some(*path))
        .cloned()
        .collect()
}

pub(crate) fn record_opened(path: PathBuf, cx: &mut App) -> bool {
    let recent = cx.global::<RecentProjects>();
    if recent.current.as_ref() == Some(&path) && recent.paths.first() == Some(&path) {
        return false;
    }
    make_current(path, cx);
    true
}

fn make_current(path: PathBuf, cx: &mut App) {
    let limit = RECENT_LIMIT.get(cx);
    let recent = cx.global_mut::<RecentProjects>();
    recent.current = Some(path.clone());
    if recent.paths.first() == Some(&path) {
        return;
    }
    recent.paths.retain(|recent_path| *recent_path != path);
    recent.paths.insert(0, path);
    recent.paths.truncate(limit);
    let paths = recent.paths.clone();
    save(paths, cx);
}

pub(crate) fn forget(path: &Path, cx: &mut App) {
    let recent = cx.global_mut::<RecentProjects>();
    recent.paths.retain(|recent_path| recent_path != path);
    let paths = recent.paths.clone();
    save(paths, cx);
}

pub(crate) fn clear(cx: &mut App) {
    let recent = cx.global_mut::<RecentProjects>();
    recent.paths = recent.current.iter().cloned().collect();
    let paths = recent.paths.clone();
    save(paths, cx);
}

fn save(paths: Vec<PathBuf>, cx: &App) {
    if let Some(file) = state_file(FILE_NAME) {
        WRITER.save(file, paths, cx);
    }
}

pub(crate) fn display_path(path: &Path) -> String {
    match dirs::home_dir().and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf)) {
        Some(relative) if relative.as_os_str().is_empty() => "~".into(),
        Some(relative) => format!("~{}{}", std::path::MAIN_SEPARATOR, relative.display()),
        None => path.display().to_string(),
    }
}

pub(crate) fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
