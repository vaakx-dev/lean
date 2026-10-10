use std::rc::Rc;

use gpui_kit::{Action, App, Global, KeyBinding, KeyBindingContextPredicate, SharedString};

use super::apply::apply;
use super::names::command_title;

pub(super) type Scope = Option<Rc<KeyBindingContextPredicate>>;

#[derive(Default)]
pub(super) struct Registry {
    pub(super) commands: Vec<Command>,
}

impl Global for Registry {}

pub(super) struct Command {
    pub(super) id: SharedString,
    pub(super) title: SharedString,
    pub(super) action: Box<dyn Action>,
    pub(super) defaults: Vec<DefaultKeys>,
}

pub(super) struct DefaultKeys {
    pub(super) keys: SharedString,
    pub(super) context: Scope,
}

impl Registry {
    pub(super) fn command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.id == id)
    }

    fn add(&mut self, id: SharedString, title: SharedString, binding: KeyBinding) {
        let defaults = DefaultKeys {
            keys: keystrokes_text(&binding).into(),
            context: binding.predicate(),
        };
        match self.commands.iter_mut().find(|command| command.id == id) {
            Some(command) => command.defaults.push(defaults),
            None => self.commands.push(Command {
                id,
                title,
                action: binding.action().boxed_clone(),
                defaults: vec![defaults],
            }),
        }
    }
}

impl Command {
    pub(super) fn default_keys(&self) -> Vec<SharedString> {
        let mut keys: Vec<SharedString> = Vec::new();
        for default in &self.defaults {
            if !keys.contains(&default.keys) {
                keys.push(default.keys.clone());
            }
        }
        keys
    }

    pub(super) fn contexts(&self) -> Vec<Scope> {
        let mut contexts: Vec<Scope> = Vec::new();
        for default in &self.defaults {
            if !contexts.contains(&default.context) {
                contexts.push(default.context.clone());
            }
        }
        if contexts.is_empty() {
            contexts.push(None);
        }
        contexts
    }
}

pub(crate) fn bind_defaults(cx: &mut App, bindings: impl IntoIterator<Item = KeyBinding>) {
    let registry = cx.default_global::<Registry>();
    for binding in bindings {
        let id = SharedString::from(binding.action().name());
        let title = command_title(&id);
        registry.add(id, title, binding);
    }
    apply(cx);
}

pub(crate) fn bind_command(
    cx: &mut App,
    id: &'static str,
    title: &'static str,
    bindings: impl IntoIterator<Item = KeyBinding>,
) {
    let registry = cx.default_global::<Registry>();
    for binding in bindings {
        registry.add(id.into(), title.into(), binding);
    }
    apply(cx);
}

fn keystrokes_text(binding: &KeyBinding) -> String {
    binding
        .keystrokes()
        .iter()
        .map(|keystroke| keystroke.unparse())
        .collect::<Vec<_>>()
        .join(" ")
}
