use std::collections::BTreeMap;
use std::time::Duration;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::*;

use super::ENVIRONMENT;
use crate::settings::ui::fields::Field;

const TYPING_DELAY: Duration = Duration::from_millis(600);
const STATE_ID: &str = "terminal-environment";

type Environment = BTreeMap<String, String>;

struct EnvironmentEditor {
    input: Entity<TextareaState>,
    synced: Environment,
    _pending: Task<()>,
    _subscription: Subscription,
}

pub(super) fn environment_field() -> Field {
    Field::bound(
        &ENVIRONMENT,
        "Environment variables",
        "One NAME=value per line, set in every new terminal.",
        |options, window, cx| {
            let input = environment_input(window, cx);
            Textarea::new(&input)
                .disabled(options.is_disabled())
                .with_size(options.size())
                .w_full()
                .into_any_element()
        },
    )
    .stacked()
    .keywords(["env", "variables", "PATH"])
}

fn environment_input(window: &mut Window, cx: &mut App) -> Entity<TextareaState> {
    let value = ENVIRONMENT.get(cx);
    let initial = value.clone();
    let editor = window.use_keyed_state(STATE_ID, cx, move |window, cx| {
        EnvironmentEditor::new(initial, window, cx)
    });
    editor.update(cx, |editor, cx| editor.sync(value, window, cx));
    editor.read(cx).input.clone()
}

impl EnvironmentEditor {
    fn new(value: Environment, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = format_environment(&value);
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(2, 8)
                .placeholder("NAME=value")
                .default_value(text)
        });
        let subscription = cx.subscribe(&input, |editor, _, event: &InputEvent, cx| {
            editor.handle_input_event(event, cx)
        });
        Self {
            input,
            synced: value,
            _pending: Task::ready(()),
            _subscription: subscription,
        }
    }

    fn handle_input_event(&mut self, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Change => self.schedule_commit(cx),
            InputEvent::Blur => self.commit(cx),
            InputEvent::PressEnter { .. } | InputEvent::Focus => {}
        }
    }

    fn schedule_commit(&mut self, cx: &mut Context<Self>) {
        self._pending = cx.spawn(async move |editor, cx| {
            cx.background_executor().timer(TYPING_DELAY).await;
            editor.update(cx, |editor, cx| editor.commit(cx)).ok();
        });
    }

    fn commit(&mut self, cx: &mut Context<Self>) {
        self._pending = Task::ready(());
        let environment = parse_environment(&self.input.read(cx).value());
        if environment != self.synced {
            ENVIRONMENT.set(environment, cx);
        }
    }

    fn sync(&mut self, value: Environment, window: &mut Window, cx: &mut Context<Self>) {
        if value == self.synced {
            return;
        }
        let text = format_environment(&value);
        let current = parse_environment(&self.input.read(cx).value());
        self.synced = value;
        if current != self.synced {
            self._pending = Task::ready(());
            self.input
                .update(cx, |input, cx| input.set_value(text, window, cx));
        }
    }
}

fn parse_environment(text: &str) -> Environment {
    text.lines()
        .filter_map(|line| {
            let (name, value) = line.split_once('=')?;
            let name = name.trim();
            (!name.is_empty()).then(|| (name.to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn format_environment(environment: &Environment) -> SharedString {
    environment
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("\n")
        .into()
}
