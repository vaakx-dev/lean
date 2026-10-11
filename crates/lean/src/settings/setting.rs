use gpui_kit::{App, Context, Subscription, Window};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::global::{Settings, read, write};

pub(crate) trait SettingValue:
    Serialize + DeserializeOwned + Clone + PartialEq + 'static
{
}

impl<T: Serialize + DeserializeOwned + Clone + PartialEq + 'static> SettingValue for T {}

#[derive(Clone)]
pub(crate) struct Setting<T> {
    pub(crate) key: &'static str,
    default: Fallback<T>,
}

#[derive(Clone)]
enum Fallback<T> {
    Value(T),
    Make(fn() -> T),
}

impl<T> Setting<T> {
    pub(crate) const fn new(key: &'static str, default: T) -> Self {
        Self {
            key,
            default: Fallback::Value(default),
        }
    }

    pub(crate) const fn new_with(key: &'static str, default: fn() -> T) -> Self {
        Self {
            key,
            default: Fallback::Make(default),
        }
    }
}

impl<T: SettingValue> Setting<T> {
    pub(crate) fn default_value(&self) -> T {
        match &self.default {
            Fallback::Value(value) => value.clone(),
            Fallback::Make(make) => make(),
        }
    }

    pub(crate) fn get(&self, cx: &App) -> T {
        read(self.key, cx)
            .and_then(|value| <T as Deserialize>::deserialize(value).ok())
            .unwrap_or_else(|| self.default_value())
    }

    pub(crate) fn set(&self, value: T, cx: &mut App) {
        if value == self.default_value() {
            write(self.key, None, cx);
            return;
        }
        match serde_json::to_value(&value) {
            Ok(json) => write(self.key, Some(json), cx),
            Err(error) => log::warn!("failed to store setting {}: {error}", self.key),
        }
    }

    pub(crate) fn reset(&self, cx: &mut App) {
        write(self.key, None, cx);
    }

    pub(crate) fn is_modified(&self, cx: &App) -> bool {
        self.get(cx) != self.default_value()
    }

    pub(crate) fn observe<V: 'static>(
        &self,
        cx: &mut Context<V>,
        mut on_change: impl FnMut(&mut V, T, &mut Context<V>) + 'static,
    ) -> Subscription {
        let mut changed = self.changes(cx);
        cx.observe_global::<Settings>(move |view, cx| {
            if let Some(value) = changed(cx) {
                on_change(view, value, cx);
            }
        })
    }

    pub(crate) fn observe_app(
        &self,
        cx: &mut App,
        mut on_change: impl FnMut(T, &mut App) + 'static,
    ) -> Subscription {
        let mut changed = self.changes(cx);
        cx.observe_global::<Settings>(move |cx| {
            if let Some(value) = changed(cx) {
                on_change(value, cx);
            }
        })
    }

    pub(crate) fn observe_in<V: 'static>(
        &self,
        window: &Window,
        cx: &mut Context<V>,
        mut on_change: impl FnMut(&mut V, T, &mut Window, &mut Context<V>) + 'static,
    ) -> Subscription {
        let mut changed = self.changes(cx);
        cx.observe_global_in::<Settings>(window, move |view, window, cx| {
            if let Some(value) = changed(cx) {
                on_change(view, value, window, cx);
            }
        })
    }

    fn changes(&self, cx: &App) -> impl FnMut(&App) -> Option<T> + use<T> {
        let setting = self.clone();
        let mut current = setting.get(cx);
        move |cx| {
            let value = setting.get(cx);
            if value == current {
                return None;
            }
            current = value.clone();
            Some(value)
        }
    }
}
