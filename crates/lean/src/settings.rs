mod global;
mod load;
mod setting;
pub(crate) mod ui;
mod watch;

use std::path::PathBuf;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, Window};
use serde_json::Value;

use crate::storage::{QueuedWriter, config_file};
use global::{read, write};

pub(crate) use global::Settings;
pub(crate) use setting::{Setting, SettingValue};
pub(crate) use ui::{open_keybindings, open_settings};

const FILE_NAME: &str = "settings.json";

static WRITER: QueuedWriter = QueuedWriter::new();

pub(crate) fn init(cx: &mut App) {
    let file = settings_file();
    let (values, error) = file.as_deref().map(load::load_initial).unwrap_or_default();
    cx.set_global(Settings::new(values, error));
    if let Some(file) = file {
        watch::start(file, cx);
    }
}

pub(crate) fn save_now() {
    WRITER.flush();
}

pub(crate) fn settings_file() -> Option<PathBuf> {
    config_file(FILE_NAME)
}

pub(crate) fn notify_load_error(window: &mut Window, cx: &mut App) {
    let Some(error) = cx
        .try_global::<Settings>()
        .and_then(|settings| settings.load_error.clone())
    else {
        return;
    };
    window.push_notification(Notification::warning(error).autohide(false), cx);
}

pub(crate) fn plugin_enabled(id: &str, cx: &App) -> bool {
    read(&plugin_key(id), cx)
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

pub(crate) fn set_plugin_enabled(id: &str, enabled: bool, cx: &mut App) {
    let value = (!enabled).then_some(Value::Bool(false));
    write(&plugin_key(id), value, cx);
}

fn plugin_key(id: &str) -> String {
    format!("plugins.{id}")
}
