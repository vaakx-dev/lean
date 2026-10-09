use std::path::Path;

use gpui_kit::{App, Window};

use super::{FileAction, Plugin, PluginContext};

pub(super) const PLUGIN: Plugin = Plugin {
    id: "browser",
    name: "Open in Browser",
    description: "Open HTML files in your default web browser.",
    activate,
};

const OPEN_IN_BROWSER: FileAction = FileAction {
    id: "open-in-browser",
    label: "Open in Browser",
    icon: "icons/globe.svg",
    applies: is_html,
    run: open_in_browser,
};

fn activate(context: &mut PluginContext, _: &mut App) {
    context.register_file_action(OPEN_IN_BROWSER);
}

fn is_html(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
    })
}

fn open_in_browser(path: &Path, _: &mut Window, cx: &mut App) {
    cx.open_with_system(path);
}
