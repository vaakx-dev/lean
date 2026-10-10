mod launch;
pub(crate) mod settings;

use std::path::Path;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, Window};

use super::{FileAction, Plugin, PluginContext};
use launch::launch;
use settings::{COMMAND, EXTENSIONS};

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
    applies: has_browser_extension,
    run: open_in_browser,
};

fn activate(context: &mut PluginContext, _: &mut App) {
    context.register_file_action(OPEN_IN_BROWSER);
}

fn has_browser_extension(path: &Path, cx: &App) -> bool {
    let Some(extension) = path.extension() else {
        return false;
    };
    EXTENSIONS
        .get(cx)
        .iter()
        .any(|wanted| extension.eq_ignore_ascii_case(wanted.trim().trim_start_matches('.')))
}

fn open_in_browser(path: &Path, window: &mut Window, cx: &mut App) {
    let command = COMMAND.get(cx);
    if command.is_empty() {
        cx.open_with_system(path);
        return;
    }
    if let Err(error) = launch(&command, path) {
        log::warn!("failed to run {command}: {error}");
        window.push_notification(
            Notification::error(format!("Couldn't open the browser {command}: {error}")),
            cx,
        );
    }
}
