use gpui_kit::{
    App, Global, KeyBinding, KeyBindingContextPredicate, Keystroke, NoAction, Subscription,
    is_no_action,
};

use super::keymap::KEY_CONTEXT;
use crate::terminal::settings::PASSTHROUGH_KEYS;

struct PassthroughWatch {
    _subscription: Subscription,
}

impl Global for PassthroughWatch {}

pub(super) fn bind_passthrough_keys(cx: &mut App) {
    let keys = PASSTHROUGH_KEYS.get(cx);
    cx.bind_keys(passthrough_bindings(&keys));
}

pub(super) fn watch_passthrough_keys(cx: &mut App) {
    if cx.has_global::<PassthroughWatch>() {
        return;
    }
    let subscription = PASSTHROUGH_KEYS.observe_app(cx, |keys, cx| rebind(&keys, cx));
    cx.set_global(PassthroughWatch {
        _subscription: subscription,
    });
}

fn rebind(keys: &[String], cx: &mut App) {
    let context = match KeyBindingContextPredicate::parse(KEY_CONTEXT) {
        Ok(context) => context,
        Err(error) => {
            log::warn!("invalid terminal key context: {error:#}");
            return;
        }
    };
    let kept: Vec<KeyBinding> = cx
        .key_bindings()
        .borrow()
        .bindings()
        .filter(|binding| !is_passthrough(binding, &context))
        .cloned()
        .collect();
    cx.clear_key_bindings();
    cx.bind_keys(kept.into_iter().chain(passthrough_bindings(keys)));
}

fn is_passthrough(binding: &KeyBinding, context: &KeyBindingContextPredicate) -> bool {
    is_no_action(binding.action()) && binding.predicate().as_deref() == Some(context)
}

fn passthrough_bindings(keys: &[String]) -> Vec<KeyBinding> {
    keys.iter()
        .map(|keys| keys.trim())
        .filter(|keys| is_valid(keys))
        .map(|keys| KeyBinding::new(keys, NoAction, Some(KEY_CONTEXT)))
        .collect()
}

fn is_valid(keys: &str) -> bool {
    let valid = !keys.is_empty()
        && keys
            .split_whitespace()
            .all(|keystroke| Keystroke::parse(keystroke).is_ok());
    if !valid {
        log::warn!("ignoring invalid terminal passthrough key {keys:?}");
    }
    valid
}
