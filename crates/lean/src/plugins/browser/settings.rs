use gpui_kit::App;
use gpui_kit::component::setting::SettingGroup;

use crate::settings::Setting;
use crate::settings::ui::fields::{PathKind, group, list, path};

pub(crate) const COMMAND: Setting<String> = Setting::new("browser.command", String::new());
pub(crate) const EXTENSIONS: Setting<Vec<String>> =
    Setting::new_with("browser.extensions", default_extensions);

fn default_extensions() -> Vec<String> {
    vec!["html".into(), "htm".into()]
}

pub(crate) fn settings_groups(_: &App) -> Vec<SettingGroup> {
    vec![group(
        "Open in Browser",
        [
            path(
                &COMMAND,
                "Browser",
                "Program that opens the file. Empty uses the system default (xdg-open).",
                PathKind::File,
                "xdg-open",
            )
            .keywords(["command", "firefox", "chrome", "xdg-open"]),
            list(
                &EXTENSIONS,
                "File extensions",
                "Files with these extensions get an Open in Browser action.",
                "html, htm",
            )
            .keywords(["html", "htm", "svg"]),
        ],
    )]
}
