use std::collections::BTreeMap;

use gpui_kit::{App, SharedString};
use serde_json::{Map, Value};

use super::registry::Registry;
use crate::settings::Setting;

pub(crate) const KEYMAP: Setting<Map<String, Value>> = Setting::new_with("keymap", Map::new);

pub(super) enum Override {
    Unbound,
    Keys(Vec<SharedString>),
}

pub(super) fn overrides(cx: &App) -> BTreeMap<String, Override> {
    KEYMAP
        .get(cx)
        .into_iter()
        .filter_map(|(id, value)| Some((id, parse(&value)?)))
        .collect()
}

pub(super) fn invalid_entries(cx: &App) -> Vec<String> {
    KEYMAP
        .get(cx)
        .into_iter()
        .filter(|(_, value)| parse(value).is_none())
        .map(|(id, _)| id)
        .collect()
}

fn parse(value: &Value) -> Option<Override> {
    match value {
        Value::Null => Some(Override::Unbound),
        Value::String(keys) => Some(keys_override(vec![keys.as_str()])),
        Value::Array(items) => items
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()
            .map(keys_override),
        _ => None,
    }
}

fn keys_override(keys: Vec<&str>) -> Override {
    let keys: Vec<SharedString> = keys
        .into_iter()
        .map(|sequence| sequence.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|sequence| !sequence.is_empty())
        .map(SharedString::from)
        .collect();
    if keys.is_empty() {
        Override::Unbound
    } else {
        Override::Keys(keys)
    }
}

pub(crate) fn set_keys(id: &str, keys: Vec<SharedString>, cx: &mut App) {
    if keys.is_empty() {
        remove_keys(id, cx);
        return;
    }
    if default_keys(id, cx).is_some_and(|defaults| defaults == keys) {
        reset_keys(id, cx);
        return;
    }
    let value = match keys.as_slice() {
        [sequence] => Value::String(sequence.to_string()),
        sequences => Value::Array(
            sequences
                .iter()
                .map(|sequence| Value::String(sequence.to_string()))
                .collect(),
        ),
    };
    write(id, Some(value), cx);
}

pub(crate) fn remove_keys(id: &str, cx: &mut App) {
    if default_keys(id, cx).is_some_and(|defaults| !defaults.is_empty()) {
        write(id, Some(Value::Null), cx);
    } else {
        reset_keys(id, cx);
    }
}

pub(crate) fn reset_keys(id: &str, cx: &mut App) {
    write(id, None, cx);
}

fn write(id: &str, value: Option<Value>, cx: &mut App) {
    let mut keymap = KEYMAP.get(cx);
    match value {
        Some(value) => keymap.insert(id.to_owned(), value),
        None => keymap.remove(id),
    };
    KEYMAP.set(keymap, cx);
}

fn default_keys(id: &str, cx: &App) -> Option<Vec<SharedString>> {
    cx.try_global::<Registry>()?
        .command(id)
        .map(|command| command.default_keys())
}
