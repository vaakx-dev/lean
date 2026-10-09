use gpui_kit::{App, Menu, MenuItem};

use super::{display_path, others};
use crate::app::actions::{ClearRecentProjects, OpenRecent, OpenRecentProject};

const MENU_LIMIT: usize = 10;

pub(crate) fn open_recent_menu(cx: &App) -> Menu {
    let projects = others(cx);
    let mut items: Vec<MenuItem> = projects
        .iter()
        .take(MENU_LIMIT)
        .map(|path| MenuItem::action(display_path(path), OpenRecentProject { path: path.clone() }))
        .collect();
    if !items.is_empty() {
        items.push(MenuItem::separator());
    }
    items.push(MenuItem::action("More…", OpenRecent));
    items.push(
        MenuItem::action("Clear Recently Opened", ClearRecentProjects)
            .disabled(projects.is_empty()),
    );
    Menu::new("Open Recent").items(items)
}
