use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

const TYPING_DELAY: Duration = Duration::from_millis(600);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Commit {
    Typing,
    Final,
}

pub(super) struct DraftInput {
    pub(super) key: &'static str,
    pub(super) value: SharedString,
    pub(super) normalize: fn(&str) -> SharedString,
}

struct Draft {
    input: Entity<InputState>,
    synced: SharedString,
    normalize: fn(&str) -> SharedString,
    commit: Rc<dyn Fn(&str, Commit, &mut App)>,
    pending: Task<()>,
    _subscription: Subscription,
}

impl DraftInput {
    pub(super) fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        configure: impl FnOnce(InputState) -> InputState,
        commit: impl Fn(&str, Commit, &mut App) + 'static,
    ) -> Entity<InputState> {
        let id = SharedString::from(format!("setting-draft-{}", self.key));
        let value = self.value.clone();
        let draft = window.use_keyed_state(id, cx, |window, cx| {
            let input = cx.new(|cx| configure(InputState::new(window, cx)).default_value(value));
            let subscription = cx
                .subscribe(&input, |draft: &mut Draft, _, event: &InputEvent, cx| {
                    draft.handle_input_event(event, cx)
                });
            Draft {
                input,
                synced: self.value.clone(),
                normalize: self.normalize,
                commit: Rc::new(commit),
                pending: Task::ready(()),
                _subscription: subscription,
            }
        });
        draft.update(cx, |draft, cx| draft.sync(self.value, window, cx));
        draft.read(cx).input.clone()
    }
}

impl Draft {
    fn handle_input_event(&mut self, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Change => self.schedule(cx),
            InputEvent::PressEnter { .. } | InputEvent::Blur => self.commit(Commit::Final, cx),
            InputEvent::Focus => {}
        }
    }

    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.pending = cx.spawn(async move |draft, cx| {
            cx.background_executor().timer(TYPING_DELAY).await;
            draft
                .update(cx, |draft, cx| draft.commit(Commit::Typing, cx))
                .ok();
        });
    }

    fn commit(&mut self, kind: Commit, cx: &mut Context<Self>) {
        self.pending = Task::ready(());
        let text = self.input.read(cx).value();
        if (self.normalize)(&text) == self.synced {
            return;
        }
        let commit = self.commit.clone();
        commit(&text, kind, cx);
    }

    fn sync(&mut self, value: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if value == self.synced {
            return;
        }
        self.synced = value.clone();
        let text = self.input.read(cx).value();
        if (self.normalize)(&text) != value {
            self.pending = Task::ready(());
            self.input
                .update(cx, |input, cx| input.set_value(value, window, cx));
        }
    }
}
