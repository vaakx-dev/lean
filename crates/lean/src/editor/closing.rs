use std::path::Path;

use gpui_kit::*;

use super::{EditorArea, EditorAreaEvent};

const SAVE_ANSWER: usize = 0;
const DISCARD_ANSWER: usize = 1;

impl EditorArea {
    pub(crate) fn close_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(active) = self.active {
            self.close(active, window, cx);
        }
    }

    pub(super) fn close(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(file) = self.files.get(index) else {
            return;
        };
        if !file.dirty {
            self.remove(index, window, cx);
            return;
        }
        let path = file.path.clone();
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("Do you want to save the changes you made to {}?", file.name),
            Some("Your changes will be lost if you don't save them."),
            &[
                PromptButton::ok("Save"),
                PromptButton::new("Don't Save"),
                PromptButton::cancel("Cancel"),
            ],
            cx,
        );
        cx.spawn_in(window, async move |area, cx| {
            let answer = match answer.await {
                Ok(answer) => answer,
                Err(error) => {
                    log::debug!("the save prompt was dismissed: {error}");
                    return;
                }
            };
            let update = area.update_in(cx, |area, window, cx| {
                area.finish_close(&path, answer, window, cx)
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while closing a file: {error:#}");
            }
        })
        .detach();
    }

    fn finish_close(
        &mut self,
        path: &Path,
        answer: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.index_of(path) else {
            return;
        };
        match answer {
            SAVE_ANSWER => {
                if self.save(index, window, cx) {
                    self.remove(index, window, cx);
                }
            }
            DISCARD_ANSWER => self.remove(index, window, cx),
            _ => {}
        }
    }

    fn remove(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.files.len() {
            return;
        }
        self.files.remove(index);
        match self.active {
            _ if self.files.is_empty() => {
                self.active = None;
                window.focus(&self.focus_handle, cx);
                cx.emit(EditorAreaEvent::ActiveEditorChanged);
                cx.notify();
            }
            Some(active) if active >= index => {
                let next = active.saturating_sub(1).min(self.files.len() - 1);
                self.activate(next, window, cx);
            }
            _ => cx.notify(),
        }
    }
}
