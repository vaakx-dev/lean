use gpui_kit::*;

use super::super::EditorArea;
use super::super::disk_state::SaveOutcome;
use super::settle;
use crate::files::settings::SaveFormat;

const OVERWRITE_ANSWER: usize = 0;
const REVERT_ANSWER: usize = 1;

impl EditorArea {
    pub(super) fn resolve_conflict(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let Some(file) = self.files.get(index) else {
            return Task::ready(false);
        };
        let id = file.id;
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("{} has changed on disk.", file.name),
            Some("Overwrite it with your changes, or revert to the version on disk?"),
            &[
                PromptButton::ok("Overwrite"),
                PromptButton::new("Revert"),
                PromptButton::cancel("Cancel"),
            ],
            cx,
        );
        cx.spawn_in(window, async move |area, cx| {
            let answer = match answer.await {
                Ok(answer) => answer,
                Err(error) => {
                    log::debug!("the conflict prompt was dismissed: {error}");
                    return false;
                }
            };
            let next = area.update_in(cx, |area, window, cx| {
                let index = area.index_of_id(id)?;
                Some(match answer {
                    OVERWRITE_ANSWER => area.overwrite(index, window, cx),
                    REVERT_ANSWER => {
                        area.revert(index, window, cx);
                        Task::ready(true)
                    }
                    _ => Task::ready(false),
                })
            });
            settle(next).await
        })
    }

    fn overwrite(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let write = self.write(index, SaveFormat::from_settings(cx), window, cx);
        cx.spawn(async move |_, _| write.await == SaveOutcome::Saved)
    }
}
