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
        let id = file.id;
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
                area.finish_close(id, answer, window, cx)
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while closing a file: {error:#}");
            }
        })
        .detach();
    }

    fn finish_close(
        &mut self,
        id: u64,
        answer: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.index_of_id(id) else {
            return;
        };
        match answer {
            SAVE_ANSWER => self.save_and_remove(index, window, cx),
            DISCARD_ANSWER => self.remove(index, window, cx),
            _ => {}
        }
    }

    fn save_and_remove(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.files.get(index).map(|file| file.id) else {
            return;
        };
        let save = self.save(index, window, cx);
        cx.spawn_in(window, async move |area, cx| {
            if !save.await {
                return;
            }
            let update = area.update_in(cx, |area, window, cx| {
                if let Some(index) = area.index_of_id(id) {
                    area.remove(index, window, cx);
                }
            });
            if let Err(error) = update {
                log::debug!("editor area dropped while closing a saved file: {error:#}");
            }
        })
        .detach();
    }

    pub(super) fn remove(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.files.len() {
            return;
        }
        self.files.remove(index);
        cx.emit(EditorAreaEvent::OpenFilesChanged);
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
