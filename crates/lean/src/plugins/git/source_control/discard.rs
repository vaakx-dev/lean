use std::rc::Rc;

use gpui_kit::*;

use super::SourceControl;
use super::changes::{ChangedFile, paths_of};
use crate::plugins::git::repo::FileStatus;
use crate::plugins::git::settings::CONFIRM_DISCARD;

impl SourceControl {
    pub(super) fn confirm_discard(
        &mut self,
        files: Vec<Rc<ChangedFile>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if files.is_empty() {
            return;
        }
        if !CONFIRM_DISCARD.get(cx) {
            self.discard(paths_of(&files), window, cx);
            return;
        }
        let (message, confirm_label) = discard_question(&files);
        let answer = window.prompt(
            PromptLevel::Warning,
            &message,
            Some("This action is irreversible."),
            &[confirm_label, "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |source_control, cx| {
            if !matches!(answer.await, Ok(0)) {
                return;
            }
            let paths = paths_of(&files);
            let update = source_control.update_in(cx, |source_control, window, cx| {
                source_control.discard(paths, window, cx)
            });
            if let Err(error) = update {
                log::debug!("source control closed before discarding changes: {error:#}");
            }
        })
        .detach();
    }
}

fn discard_question(files: &[Rc<ChangedFile>]) -> (String, &'static str) {
    match files {
        [file] if file.status == FileStatus::Untracked => (
            format!("Are you sure you want to delete \"{}\"?", file.name),
            "Delete File",
        ),
        [file] => (
            format!(
                "Are you sure you want to discard the changes in \"{}\"?",
                file.name
            ),
            "Discard Changes",
        ),
        files => (
            format!(
                "Are you sure you want to discard the changes in {} files?",
                files.len()
            ),
            "Discard All Changes",
        ),
    }
}
