use std::path::PathBuf;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::Textarea;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, WindowExt as _, v_flex};
use gpui_kit::*;

use super::SourceControl;
use super::changes::{ChangeGroup, paths_of};
use super::keymap::{COMMIT_MESSAGE_CONTEXT, Commit};
use crate::plugins::git::repo as git;
use crate::plugins::git::settings::{NOTHING_STAGED, NothingStaged};

impl SourceControl {
    pub(super) fn render_commit_box(&self, cx: &Context<Self>) -> Div {
        let busy = self.is_busy();
        v_flex()
            .flex_shrink_0()
            .px_2()
            .pb_2()
            .gap_2()
            .child(
                div()
                    .key_context(COMMIT_MESSAGE_CONTEXT)
                    .on_action(cx.listener(Self::commit))
                    .child(Textarea::new(&self.commit_message).small()),
            )
            .child(
                Button::new("commit")
                    .primary()
                    .small()
                    .w_full()
                    .icon(Icon::empty().path("icons/check.svg"))
                    .label("Commit")
                    .loading(busy)
                    .disabled(busy)
                    .on_click(cx.listener(|source_control, _, window, cx| {
                        source_control.commit(&Commit, window, cx)
                    })),
            )
    }

    fn commit(&mut self, _: &Commit, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_busy() {
            return;
        }
        let message = self.commit_message.read(cx).value().trim().to_string();
        if message.is_empty() {
            window.push_notification(Notification::warning("Enter a commit message first."), cx);
            self.commit_message
                .update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        if !self.changes.files(ChangeGroup::Staged).is_empty() {
            self.start_commit(message, Vec::new(), window, cx);
            return;
        }
        if self.changes.files(ChangeGroup::Unstaged).is_empty() {
            window.push_notification(Notification::info("There are no changes to commit."), cx);
            return;
        }
        match NOTHING_STAGED.get(cx) {
            NothingStaged::Ask => self.offer_to_commit_everything(message, window, cx),
            NothingStaged::StageAll => {
                let paths = paths_of(self.changes.files(ChangeGroup::Unstaged));
                self.start_commit(message, paths, window, cx);
            }
            NothingStaged::DoNothing => window.push_notification(
                Notification::info("There are no staged changes to commit."),
                cx,
            ),
        }
    }

    fn offer_to_commit_everything(
        &mut self,
        message: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let answer = window.prompt(
            PromptLevel::Info,
            "There are no staged changes to commit.",
            Some("Would you like to stage all your changes and commit them directly?"),
            &["Stage All and Commit", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |source_control, cx| {
            if !matches!(answer.await, Ok(0)) {
                return;
            }
            source_control
                .update_in(cx, |source_control, window, cx| {
                    let paths = paths_of(source_control.changes.files(ChangeGroup::Unstaged));
                    source_control.start_commit(message, paths, window, cx)
                })
                .ok();
        })
        .detach();
    }

    fn start_commit(
        &mut self,
        message: String,
        paths_to_stage: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_git_operation(
            "commit",
            move |root| git::commit(root, &message, &paths_to_stage),
            |source_control, window, cx| {
                source_control
                    .commit_message
                    .update(cx, |input, cx| input.set_value("", window, cx))
            },
            window,
            cx,
        );
    }
}
