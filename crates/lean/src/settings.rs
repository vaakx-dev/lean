mod global;
mod load;
mod plugins;
mod save;
mod setting;
pub(crate) mod ui;
mod watch;

use std::path::PathBuf;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, SharedString, Window};

use crate::storage::config_file;

pub(crate) use global::Settings;
pub(crate) use plugins::{plugin_enabled, set_plugin_enabled};
pub(crate) use setting::{Setting, SettingValue};
pub(crate) use ui::{open_keybindings, open_settings};

const FILE_NAME: &str = "settings.json";

pub(crate) fn init(cx: &mut App) {
    let file = settings_file();
    let loaded = file.as_deref().map(load::load_initial);
    let (values, error) = loaded
        .map(|loaded| (loaded.values, loaded.error))
        .unwrap_or_default();
    cx.set_global(Settings::new(values, error));
    cx.on_app_quit(|cx| {
        save_now(cx);
        async {}
    })
    .detach();
    if let Some(file) = file {
        watch::start(file, cx);
    }
}

pub(crate) fn save_now(cx: &App) {
    save::flush_now(cx);
}

pub(crate) fn settings_file() -> Option<PathBuf> {
    config_file(FILE_NAME)
}

pub(crate) fn load_error(cx: &App) -> Option<SharedString> {
    cx.try_global::<Settings>()?.load_error.clone()
}

pub(crate) fn notify_load_error(window: &mut Window, cx: &mut App) {
    if let Some(error) = load_error(cx) {
        window.push_notification(Notification::warning(error).autohide(false), cx);
    }
}
