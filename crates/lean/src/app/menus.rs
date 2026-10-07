use gpui_kit::component::{GlobalState, input};
use gpui_kit::{App, Menu, MenuItem};

use super::actions::{
    About, CloseActiveEditor, NextEditor, OpenFolder, PreviousEditor, Quit, SaveActiveEditor,
    ShowExplorer, ShowSearch, ShowSourceControl, ToggleSidebar, ToggleTerminal,
};

pub(super) fn set_menus(cx: &mut App) {
    let owned_menus = app_menus().into_iter().map(Menu::owned).collect();
    GlobalState::global_mut(cx).set_app_menus(owned_menus);
    cx.set_menus(app_menus());
}

fn app_menus() -> Vec<Menu> {
    vec![
        Menu::new("File").items([
            MenuItem::action("Open Folder…", OpenFolder),
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
        Menu::new("View").items([
            MenuItem::action("Explorer", ShowExplorer),
            MenuItem::action("Search", ShowSearch),
            MenuItem::action("Source Control", ShowSourceControl),
            MenuItem::separator(),
            MenuItem::action("Toggle Sidebar", ToggleSidebar),
            MenuItem::action("Toggle Terminal", ToggleTerminal),
        ]),
        Menu::new("Go").items([
            MenuItem::action("Next Editor", NextEditor),
            MenuItem::action("Previous Editor", PreviousEditor),
        ]),
        Menu::new("Terminal").items([MenuItem::action("Toggle Terminal", ToggleTerminal)]),
        Menu::new("Help").items([MenuItem::action("About Lean", About)]),
    ]
}
