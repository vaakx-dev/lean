use gpui_kit::App;
use serde_json::Value;

use super::global::{read, write};

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
