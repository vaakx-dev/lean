mod capture;
mod keys;
mod row;
mod table;

use gpui_kit::App;
use gpui_kit::component::Icon;
#[cfg(feature = "terminal")]
use gpui_kit::component::setting::SettingGroup;
use gpui_kit::component::setting::SettingPage;

#[cfg(feature = "terminal")]
use crate::settings::ui::fields::{group, list};
#[cfg(feature = "terminal")]
use crate::terminal::settings::PASSTHROUGH_KEYS;

pub(crate) fn settings_page(cx: &App) -> SettingPage {
    let page = SettingPage::new("Keybindings")
        .icon(Icon::empty().path("icons/keyboard.svg"))
        .description("Overrides are saved to settings.json under \"keymap\".")
        .group(table::shortcuts_group(cx));
    #[cfg(feature = "terminal")]
    let page = page.group(terminal_group());
    page
}

#[cfg(feature = "terminal")]
fn terminal_group() -> SettingGroup {
    group(
        "Terminal",
        [list(
            &PASSTHROUGH_KEYS,
            "Keys sent to the terminal",
            "Shortcuts the focused terminal receives instead of Lean.",
            "ctrl-c, tab",
        )
        .stacked()
        .keywords(["passthrough", "shell", "keystrokes"])],
    )
}
