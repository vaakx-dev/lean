mod store;

use gpui_kit::{App, Global};
use serde_json::{Map, Value};

const PLUGINS_KEY: &str = "plugins";

struct Settings {
    document: Map<String, Value>,
}

impl Global for Settings {}

pub(crate) fn init(cx: &mut App) {
    cx.set_global(Settings {
        document: store::load(),
    });
}

pub(crate) fn plugin_enabled(id: &str, cx: &App) -> bool {
    cx.global::<Settings>()
        .document
        .get(PLUGINS_KEY)
        .and_then(|plugins| plugins.get(id))
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

pub(crate) fn set_plugin_enabled(id: &str, enabled: bool, cx: &mut App) {
    let settings = cx.global_mut::<Settings>();
    let plugins = settings
        .document
        .entry(PLUGINS_KEY)
        .or_insert_with(|| Value::Object(Map::new()));
    if !plugins.is_object() {
        *plugins = Value::Object(Map::new());
    }
    if let Some(plugins) = plugins.as_object_mut() {
        plugins.insert(id.to_owned(), Value::Bool(enabled));
    }
    let document = settings.document.clone();
    store::save(document, cx);
}
