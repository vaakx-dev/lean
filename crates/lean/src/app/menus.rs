use gpui_kit::component::{GlobalState, input};
use gpui_kit::{App, Menu, MenuItem};

use crate::{plugins, recent_projects};

use super::actions::{
    About, CloseActiveEditor, NextEditor, OpenFolder, PreviousEditor, Quit, SaveActiveEditor,
    ShowPanel, ToggleSidebar, ToggleTerminal,
};

pub(crate) fn set_menus(cx: &mut App) {
    cx.set_menus(app_menus(cx));
    let owned_menus = cx
        .get_menus()
        .unwrap_or_else(|| app_menus(cx).into_iter().map(Menu::owned).collect());
    GlobalState::global_mut(cx).set_app_menus(owned_menus);
}

fn app_menus(cx: &App) -> Vec<Menu> {
    vec![
        Menu::new("File").items([
            MenuItem::action("Open Folder…", OpenFolder),
            MenuItem::submenu(recent_projects::open_recent_menu(cx)),
            MenuItem::separator(),
            MenuItem::action("Save", SaveActiveEditor),
            MenuItem::action("Close Editor", CloseActiveEditor),
            MenuItem::separator(),
            MenuItem::action("Quit", Quit),
        ]),
        Menu::new("Edit").items([
            MenuItem::action("Undo", input::Undo),
            MenuItem::action("Redo", input::Redo),
            MenuItem::separator(),
            MenuItem::action("Cut", input::Cut),
            MenuItem::action("Copy", input::Copy),
            MenuItem::action("Paste", input::Paste),
            MenuItem::separator(),
            MenuItem::action("Find", input::Search),
        ]),
        Menu::new("Selection").items([MenuItem::action("Select All", input::SelectAll)]),
        Menu::new("View").items(view_items(cx)),
        Menu::new("Go").items([
            MenuItem::action("Next Editor", NextEditor),
            MenuItem::action("Previous Editor", PreviousEditor),
        ]),
        Menu::new("Terminal").items([MenuItem::action("Toggle Terminal", ToggleTerminal)]),
        Menu::new("Help").items([MenuItem::action("About Lean", About)]),
    ]
}

fn view_items(cx: &App) -> Vec<MenuItem> {
    let panels = plugins::panels(cx)
        .iter()
        .map(|panel| MenuItem::action(panel.title, ShowPanel { id: panel.id }));
    panels
        .chain([
            MenuItem::separator(),
            MenuItem::action("Toggle Sidebar", ToggleSidebar),
            MenuItem::action("Toggle Terminal", ToggleTerminal),
        ])
        .collect()
}
