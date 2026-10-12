use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use crate::app::keymap::{KEYMAP, Table, set_keys};

const SEQUENCE_DELAY: Duration = Duration::from_millis(900);
const MAX_KEYSTROKES: usize = 2;
const MODIFIER_KEYS: [&str; 5] = ["shift", "control", "alt", "platform", "function"];

#[derive(Clone, PartialEq)]
pub(super) enum Target {
    Command(SharedString),
    Search,
}

pub(super) struct Recording {
    pub(super) target: Target,
    pub(super) keystrokes: Vec<Keystroke>,
    restore_focus: Option<FocusHandle>,
}

pub(super) struct TableState {
    pub(super) search: Entity<InputState>,
    pub(super) recording: Option<Recording>,
    pub(super) focus_handle: FocusHandle,
    pub(super) table: Table,
    commit_task: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl TableState {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx
            .new(|cx| InputState::new(window, cx).placeholder("Search commands, keys or contexts"));
        let focus_handle = cx.focus_handle();
        let subscriptions = vec![
            cx.subscribe(&search, |_, _, _: &InputEvent, cx| cx.notify()),
            cx.on_blur(&focus_handle, window, |state, _, cx| state.cancel(cx)),
            intercept_keystrokes(&focus_handle, cx),
            KEYMAP.observe(cx, |state, _, cx| {
                state.table = Table::load(cx);
                cx.notify();
            }),
        ];
        Self {
            search,
            recording: None,
            focus_handle,
            table: Table::load(cx),
            commit_task: Task::ready(()),
            _subscriptions: subscriptions,
        }
    }

    pub(super) fn is_recording(&self, target: &Target) -> bool {
        self.recording
            .as_ref()
            .is_some_and(|recording| &recording.target == target)
    }

    pub(super) fn toggle(&mut self, target: Target, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_recording(&target) {
            self.finish(window, cx);
            return;
        }
        let restore_focus = self
            .recording
            .take()
            .and_then(|recording| recording.restore_focus)
            .or_else(|| window.focused(cx));
        self.commit_task = Task::ready(());
        self.recording = Some(Recording {
            target,
            keystrokes: Vec::new(),
            restore_focus,
        });
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn record(&mut self, keystroke: Keystroke, window: &mut Window, cx: &mut Context<Self>) {
        if MODIFIER_KEYS.contains(&keystroke.key.as_str()) {
            return;
        }
        let Some(recording) = self.recording.as_mut() else {
            return;
        };
        let plain = !keystroke.modifiers.modified();
        if plain && keystroke.key == "escape" {
            self.finish(window, cx);
            return;
        }
        if plain && keystroke.key == "enter" && !recording.keystrokes.is_empty() {
            self.commit(window, cx);
            return;
        }
        recording.keystrokes.push(Keystroke {
            modifiers: keystroke.modifiers,
            key: keystroke.key,
            key_char: None,
        });
        if recording.target == Target::Search || recording.keystrokes.len() >= MAX_KEYSTROKES {
            self.commit(window, cx);
        } else {
            self.schedule_commit(window, cx);
            cx.notify();
        }
    }

    fn schedule_commit(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.commit_task = cx.spawn_in(window, async move |state, cx| {
            cx.background_executor().timer(SEQUENCE_DELAY).await;
            cx.update(|window, cx| {
                state.update(cx, |state, cx| state.commit(window, cx)).ok();
            })
            .ok();
        });
    }

    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(recording) = self.recording.take() else {
            return;
        };
        self.commit_task = Task::ready(());
        let sequence = recording
            .keystrokes
            .iter()
            .map(Keystroke::unparse)
            .collect::<Vec<_>>()
            .join(" ");
        if !sequence.is_empty() {
            match &recording.target {
                Target::Command(id) => set_keys(id, vec![sequence.into()], cx),
                Target::Search => self
                    .search
                    .update(cx, |search, cx| search.set_value(sequence, window, cx)),
            }
        }
        restore_focus(recording, window, cx);
        cx.notify();
    }

    fn finish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(recording) = self.recording.take() {
            restore_focus(recording, window, cx);
        }
        self.commit_task = Task::ready(());
        cx.notify();
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        if self.recording.take().is_some() {
            self.commit_task = Task::ready(());
            cx.notify();
        }
    }
}

fn restore_focus(recording: Recording, window: &mut Window, cx: &mut App) {
    if let Some(handle) = recording.restore_focus {
        window.focus(&handle, cx);
    }
}

fn intercept_keystrokes(focus_handle: &FocusHandle, cx: &mut Context<TableState>) -> Subscription {
    let state = cx.weak_entity();
    let focus_handle = focus_handle.clone();
    cx.intercept_keystrokes(move |event, window, cx| {
        if !focus_handle.is_focused(window) {
            return;
        }
        let Some(state) = state.upgrade() else {
            return;
        };
        if state.read(cx).recording.is_none() {
            return;
        }
        let keystroke = event.keystroke.clone();
        state.update(cx, |state, cx| state.record(keystroke, window, cx));
        cx.stop_propagation();
    })
}
