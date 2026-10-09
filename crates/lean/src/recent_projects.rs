mod menu;
mod picker;
mod store;

use std::path::{Path, PathBuf};

use gpui_kit::{App, Global};

pub(crate) use menu::open_recent_menu;
pub(crate) use picker::open_picker;

const LIMIT: usize = 20;

struct RecentProjects {
    current: Option<PathBuf>,
    paths: Vec<PathBuf>,
}

impl Global for RecentProjects {}

pub(crate) fn init(project_root: PathBuf, cx: &mut App) {
    cx.set_global(RecentProjects {
        current: None,
        paths: store::load(),
    });
    make_current(project_root, cx);
}

pub(crate) fn others(cx: &App) -> Vec<PathBuf> {
    let recent = cx.global::<RecentProjects>();
    recent
        .paths
        .iter()
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
    let recent = cx.global_mut::<RecentProjects>();
    recent.current = Some(path.clone());
    if recent.paths.first() == Some(&path) {
        return;
    }
    recent.paths.retain(|recent_path| *recent_path != path);
    recent.paths.insert(0, path);
    recent.paths.truncate(LIMIT);
    let paths = recent.paths.clone();
    store::save(paths, cx);
}

pub(crate) fn forget(path: &Path, cx: &mut App) {
    let recent = cx.global_mut::<RecentProjects>();
    recent.paths.retain(|recent_path| recent_path != path);
    let paths = recent.paths.clone();
    store::save(paths, cx);
}

pub(crate) fn clear(cx: &mut App) {
    let recent = cx.global_mut::<RecentProjects>();
    recent.paths = recent.current.iter().cloned().collect();
    let paths = recent.paths.clone();
    store::save(paths, cx);
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
