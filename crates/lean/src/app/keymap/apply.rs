use std::rc::Rc;

use gpui_kit::{
    Action, App, DummyKeyboardMapper, KeyBinding, KeyBindingContextPredicate, KeyBindingMetaIndex,
    NoAction, is_no_action,
};

use super::overrides::{Override, invalid_entries, overrides};
use super::registry::{Registry, Scope};
use crate::terminal::KEY_CONTEXT;
use crate::terminal::settings::PASSTHROUGH_KEYS;

const LEAN_BINDING: KeyBindingMetaIndex = KeyBindingMetaIndex(0);

pub(super) fn apply(cx: &mut App) {
    for id in invalid_entries(cx) {
        log::warn!("ignoring keymap entry {id:?}: expected keystrokes, a list of them, or null");
    }
    let (disabling, mut bindings): (Vec<KeyBinding>, Vec<KeyBinding>) = foreign_bindings(cx)
        .into_iter()
        .partition(|binding| is_no_action(binding.action()));
    bindings.extend(lean_bindings(cx));
    bindings.extend(disabling);
    bindings.extend(passthrough_bindings(cx));
    cx.clear_key_bindings();
    cx.bind_keys(bindings);
}

fn foreign_bindings(cx: &App) -> Vec<KeyBinding> {
    let keymap = cx.key_bindings();
    let bindings: Vec<KeyBinding> = keymap
        .borrow()
        .bindings()
        .filter(|binding| binding.meta() != Some(LEAN_BINDING))
        .cloned()
        .collect();
    bindings
}

fn lean_bindings(cx: &App) -> Vec<KeyBinding> {
    let Some(registry) = cx.try_global::<Registry>() else {
        return Vec::new();
    };
    let overrides = overrides(cx);
    let mut bindings = Vec::new();
    for command in &registry.commands {
        match overrides.get(command.id.as_str()) {
            None => {
                for default in &command.defaults {
                    push(
                        &mut bindings,
                        &default.keys,
                        &*command.action,
                        &default.context,
                    );
                }
            }
            Some(Override::Unbound) => {}
            Some(Override::Keys(keys)) => {
                for context in command.contexts() {
                    for sequence in keys {
                        push(&mut bindings, sequence, &*command.action, &context);
                    }
                }
            }
        }
    }
    for (id, entry) in &overrides {
        let Override::Keys(keys) = entry else {
            continue;
        };
        if registry.command(id).is_some() {
            continue;
        }
        match cx.build_action(id, None) {
            Ok(action) => {
                for sequence in keys {
                    push(&mut bindings, sequence, &*action, &None);
                }
            }
            Err(error) => log::warn!("ignoring keymap entry {id:?}: {error}"),
        }
    }
    bindings
}

fn passthrough_bindings(cx: &App) -> Vec<KeyBinding> {
    let Ok(context) = KeyBindingContextPredicate::parse(KEY_CONTEXT) else {
        return Vec::new();
    };
    let context = Some(Rc::new(context));
    let mut bindings = Vec::new();
    for keys in PASSTHROUGH_KEYS.get(cx) {
        let keys = keys.trim();
        if !keys.is_empty() {
            push(&mut bindings, keys, &NoAction, &context);
        }
    }
    bindings
}

fn push(bindings: &mut Vec<KeyBinding>, keys: &str, action: &dyn Action, context: &Scope) {
    match KeyBinding::load(
        keys,
        action.boxed_clone(),
        context.clone(),
        false,
        None,
        &DummyKeyboardMapper,
    ) {
        Ok(binding) => bindings.push(binding.with_meta(LEAN_BINDING)),
        Err(error) => log::warn!("ignoring keybinding {keys:?}: {error}"),
    }
}
