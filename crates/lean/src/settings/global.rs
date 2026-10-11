use std::collections::BTreeMap;

use gpui_kit::{App, Global, SharedString, Task};
use serde_json::{Map, Value};

use super::{WRITER, settings_file};

pub(crate) struct Settings {
    pub(super) values: Map<String, Value>,
    pub(super) load_error: Option<SharedString>,
    pub(super) watcher: Option<Task<()>>,
}

impl Global for Settings {}

impl Settings {
    pub(super) fn new(values: Map<String, Value>, load_error: Option<SharedString>) -> Self {
        Self {
            values,
            load_error,
            watcher: None,
        }
    }
}

pub(super) fn read<'a>(key: &str, cx: &'a App) -> Option<&'a Value> {
    cx.try_global::<Settings>()?.values.get(key)
}

pub(super) fn write(key: &str, value: Option<Value>, cx: &mut App) {
    let Some(settings) = cx.try_global::<Settings>() else {
        return;
    };
    if settings.values.get(key) == value.as_ref() {
        return;
    }
    let settings = cx.global_mut::<Settings>();
    match value {
        Some(value) => settings.values.insert(key.to_owned(), value),
        None => settings.values.remove(key),
    };
    if let Some(file) = settings_file() {
        WRITER.save(file, snapshot(cx), cx);
    }
}

pub(super) fn replace_all(values: Map<String, Value>, cx: &mut App) {
    let Some(settings) = cx.try_global::<Settings>() else {
        return;
    };
    if settings.values == values && settings.load_error.is_none() {
        return;
    }
    let settings = cx.global_mut::<Settings>();
    settings.values = values;
    settings.load_error = None;
}

pub(super) fn snapshot(cx: &App) -> BTreeMap<String, Value> {
    cx.try_global::<Settings>()
        .map(|settings| {
            settings
                .values
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}
