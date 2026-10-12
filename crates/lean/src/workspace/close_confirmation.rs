use gpui_kit::*;

use super::Workspace;
use crate::app::settings::{CONFIRM_QUIT_RUNNING_WORK, CONFIRM_UNSAVED_CLOSE};

#[derive(Clone, Copy)]
pub(crate) enum CloseIntent {
    Quit,
    SwitchProject,
}

#[derive(Clone, Copy)]
enum Choice {
    SaveAll,
    Proceed,
    Cancel,
}

struct ClosePrompt {
    message: String,
    detail: String,
    buttons: Vec<PromptButton>,
    choices: Vec<Choice>,
}

impl Workspace {
    pub(crate) fn confirm_close(
        &mut self,
        intent: CloseIntent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        if self.confirming_close {
            return Task::ready(false);
        }
        let Some(prompt) = self.close_prompt(intent, cx) else {
            return Task::ready(true);
        };
        self.confirming_close = true;
        let answer = window.prompt(
            PromptLevel::Warning,
            &prompt.message,
            Some(prompt.detail.as_str()),
            prompt.buttons.as_slice(),
            cx,
        );
        let choices = prompt.choices;
        cx.spawn_in(window, async move |workspace, cx| {
            let choice = match answer.await {
                Ok(answer) => choices.get(answer).copied().unwrap_or(Choice::Cancel),
                Err(error) => {
                    log::debug!("the close prompt was dismissed: {error}");
                    Choice::Cancel
                }
            };
            let confirmed = match choice {
                Choice::SaveAll => save_unsaved(&workspace, cx).await,
                Choice::Proceed => true,
                Choice::Cancel => false,
            };
            workspace
                .update(cx, |workspace, _| workspace.confirming_close = false)
                .ok();
            confirmed
        })
    }

    pub(super) fn guard_window_close(window: &Window, cx: &Context<Self>) {
        let workspace = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            may_close_window(&workspace, window, cx)
        });
    }

    pub(super) fn close_window_handler(
        cx: &Context<Self>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let workspace = cx.weak_entity();
        move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
            if may_close_window(&workspace, window, cx) {
                window.remove_window();
            }
        }
    }

    fn allows_window_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.close_confirmed || !self.needs_close_confirmation(CloseIntent::Quit, cx) {
            self.save_window_state(window);
            return true;
        }
        let confirmation = self.confirm_close(CloseIntent::Quit, window, cx);
        cx.spawn_in(window, async move |workspace, cx| {
            if !confirmation.await {
                return;
            }
            workspace
                .update_in(cx, |workspace, window, _| {
                    workspace.close_confirmed = true;
                    workspace.save_window_state(window);
                    window.remove_window();
                })
                .ok();
        })
        .detach();
        false
    }

    fn needs_close_confirmation(&self, intent: CloseIntent, cx: &App) -> bool {
        self.confirming_close || self.close_prompt(intent, cx).is_some()
    }

    fn close_prompt(&self, intent: CloseIntent, cx: &App) -> Option<ClosePrompt> {
        let unsaved = if CONFIRM_UNSAVED_CLOSE.get(cx) {
            self.context.editor_area.read(cx).unsaved_file_names(cx)
        } else {
            Vec::new()
        };
        let processes = match intent {
            CloseIntent::Quit if !CONFIRM_QUIT_RUNNING_WORK.get(cx) => None,
            intent => self.running_processes_notice(intent, cx),
        };
        let prompt = match (unsaved.as_slice(), processes) {
            ([], None) => return None,
            ([], Some(notice)) => ClosePrompt {
                message: intent.question().into(),
                detail: notice,
                buttons: vec![
                    PromptButton::ok(intent.action_label()),
                    PromptButton::cancel("Cancel"),
                ],
                choices: vec![Choice::Proceed, Choice::Cancel],
            },
            (names, notice) => ClosePrompt {
                message: unsaved_message(names),
                detail: unsaved_detail(notice),
                buttons: vec![
                    PromptButton::ok("Save All"),
                    PromptButton::new("Don't Save"),
                    PromptButton::cancel("Cancel"),
                ],
                choices: vec![Choice::SaveAll, Choice::Proceed, Choice::Cancel],
            },
        };
        Some(prompt)
    }

    fn running_processes_notice(&self, intent: CloseIntent, cx: &App) -> Option<String> {
        let terminals = self.running_terminal_count(cx);
        let terminal_effect =
            (terminals > 0).then(|| format!("close {}", counted(terminals, "terminal")));
        let work_effects = self
            .context
            .editor_area
            .read(cx)
            .running_work(cx)
            .into_iter()
            .map(|(kind, count)| format!("stop {}", counted(count, kind)));
        let effects: Vec<String> = terminal_effect.into_iter().chain(work_effects).collect();
        if effects.is_empty() {
            return None;
        }
        Some(format!(
            "{} will {}.",
            intent.gerund(),
            effects.join(" and ")
        ))
    }
}

impl CloseIntent {
    fn question(self) -> &'static str {
        match self {
            Self::Quit => "Do you want to quit Lean?",
            Self::SwitchProject => "Do you want to switch projects?",
        }
    }

    fn action_label(self) -> &'static str {
        match self {
            Self::Quit => "Quit",
            Self::SwitchProject => "Switch Project",
        }
    }

    fn gerund(self) -> &'static str {
        match self {
            Self::Quit => "Quitting",
            Self::SwitchProject => "Switching projects",
        }
    }
}

async fn save_unsaved(workspace: &WeakEntity<Workspace>, cx: &mut AsyncWindowContext) -> bool {
    let save = workspace.update_in(cx, |workspace, window, cx| {
        workspace
            .context
            .editor_area
            .update(cx, |area, cx| area.save_unsaved(window, cx))
    });
    match save {
        Ok(save) => save.await,
        Err(_) => false,
    }
}

fn may_close_window(workspace: &WeakEntity<Workspace>, window: &mut Window, cx: &mut App) -> bool {
    workspace
        .update(cx, |workspace, cx| {
            workspace.allows_window_close(window, cx)
        })
        .unwrap_or(true)
}

fn unsaved_message(names: &[SharedString]) -> String {
    match names {
        [name] => format!("Do you want to save the changes you made to {name}?"),
        names => format!(
            "Do you want to save the changes you made to {} files?",
            names.len()
        ),
    }
}

fn unsaved_detail(notice: Option<String>) -> String {
    let warning = "Your changes will be lost if you don't save them.";
    match notice {
        Some(notice) => format!("{warning} {notice}"),
        None => warning.to_owned(),
    }
}

fn counted(count: usize, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        count => format!("{count} {noun}s"),
    }
}
