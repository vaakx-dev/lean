use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{Context, Entity, SharedString, Window};

use super::inline_edit::EditKind;
use super::name_validation::name_error;
use crate::explorer::Explorer;

impl Explorer {
    pub(super) fn handle_edit_event(
        &mut self,
        input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self
            .editing
            .as_ref()
            .is_some_and(|edit| &edit.input == input)
        {
            return;
        }
        match event {
            InputEvent::PressEnter { .. } => self.confirm_edit(window, cx),
            InputEvent::Change => self.validate_edit(cx),
            InputEvent::Blur => self.cancel_edit(window, cx),
            InputEvent::Focus => {}
        }
    }

    fn validate_edit(&mut self, cx: &mut Context<Self>) {
        let error = self.edit_error(cx);
        if let Some(edit) = self.editing.as_mut() {
            edit.error = error;
            cx.notify();
        }
    }

    fn confirm_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(error) = self.edit_error(cx) {
            if let Some(edit) = self.editing.as_mut() {
                edit.error = Some(error);
            }
            cx.notify();
            return;
        }
        let Some(edit) = self.editing.take() else {
            return;
        };
        let name = edit.input.read(cx).value().trim().to_string();
        self.close_edit(&edit, window, cx);
        let target = edit.folder.join(&name);
        match edit.kind {
            EditKind::NewFile => self.create_entry(target, false, window, cx),
            EditKind::NewFolder => self.create_entry(target, true, window, cx),
            EditKind::Rename {
                path,
                original_name,
            } => {
                if original_name != name {
                    self.rename_entry(path, target, window, cx);
                }
            }
        }
    }

    fn edit_error(&self, cx: &Context<Self>) -> Option<SharedString> {
        let edit = self.editing.as_ref()?;
        let value = edit.input.read(cx).value();
        let siblings = self
            .children
            .get(&edit.folder)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let renamed = match &edit.kind {
            EditKind::Rename { path, .. } => Some(path.as_path()),
            EditKind::NewFile | EditKind::NewFolder => None,
        };
        name_error(value.trim(), siblings, renamed)
    }
}
