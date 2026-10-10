use std::cell::Cell;
use std::collections::BTreeMap;

use gpui_kit::{App, Global, SharedString, Task};
use notify::RecommendedWatcher;
use serde_json::{Map, Value};

use super::save;

pub(crate) struct Settings {
    pub(super) values: Map<String, Value>,
    pub(super) load_error: Option<SharedString>,
    pub(super) save_pending: Cell<bool>,
    save_task: Task<()>,
    pub(super) watcher: Option<(RecommendedWatcher, Task<()>)>,
}

impl Global for Settings {}

impl Settings {
    pub(super) fn new(values: Map<String, Value>, load_error: Option<SharedString>) -> Self {
        Self {
            values,
            load_error,
            save_pending: Cell::new(false),
            save_task: Task::ready(()),
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
    let save_task = save::schedule(cx);
    let settings = cx.global_mut::<Settings>();
    match value {
        Some(value) => settings.values.insert(key.to_owned(), value),
        None => settings.values.remove(key),
    };
    settings.save_pending.set(true);
    settings.save_task = save_task;
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
