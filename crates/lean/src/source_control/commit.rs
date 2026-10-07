use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::SourceControl;
use super::changes::ChangeGroup;
use super::keymap::Commit;
use crate::git;

impl SourceControl {
    pub(super) fn commit(&mut self, _: &Commit, window: &mut Window, cx: &mut Context<Self>) {
        if self.committing {
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
            self.start_commit(message, false, window, cx);
            return;
        }
        if self.changes.files(ChangeGroup::Unstaged).is_empty() {
            window.push_notification(Notification::info("There are no changes to commit."), cx);
            return;
        }
        self.offer_to_commit_everything(message, window, cx);
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
            let update = source_control.update_in(cx, |source_control, window, cx| {
                source_control.start_commit(message, true, window, cx)
            });
            if let Err(error) = update {
                log::debug!("source control closed before committing: {error:#}");
            }
        })
        .detach();
    }

    fn start_commit(
        &mut self,
        message: String,
        stage_everything: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.committing = true;
        cx.notify();
        let operation = self.run_git_operation(
            "commit",
            move |root| git::commit(root, &message, stage_everything),
            window,
            cx,
        );
        cx.spawn_in(window, async move |source_control, cx| {
            let committed = operation.await;
            let update = source_control.update_in(cx, |source_control, window, cx| {
                source_control.committing = false;
                if committed {
                    source_control
                        .commit_message
                        .update(cx, |input, cx| input.set_value("", window, cx));
                }
                cx.notify();
            });
            if let Err(error) = update {
                log::debug!("source control closed while committing: {error:#}");
            }
        })
        .detach();
    }
}
