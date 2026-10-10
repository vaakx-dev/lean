mod capture;
mod keys;
mod row;
mod table;

use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;

use crate::settings::ui::fields::{group, list};
use crate::terminal::settings::PASSTHROUGH_KEYS;

pub(crate) fn settings_page(cx: &App) -> SettingPage {
    SettingPage::new("Keybindings")
        .icon(Icon::empty().path("icons/keyboard.svg"))
        .description("Overrides are saved to settings.json under \"keymap\".")
        .group(table::shortcuts_group(cx))
        .group(group(
            "Terminal",
            [list(
                &PASSTHROUGH_KEYS,
                "Keys sent to the terminal",
                "Shortcuts the focused terminal receives instead of Lean.",
                "ctrl-c, tab",
            )
            .stacked()
            .keywords(["passthrough", "shell", "keystrokes"])],
        ))
}
