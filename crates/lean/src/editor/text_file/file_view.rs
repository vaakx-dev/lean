use std::path::Path;
use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};
use gpui_kit::component::input::{GutterMarker, Position, Rope};
use gpui_kit::*;

use super::decoded::take_decoded;
use super::edits::{apply_edits, edit_towards};
use super::{BYTE_ORDER_MARK, TextFile};
use crate::editor::file_text::Fingerprint;
use crate::editor::saving::format_for_save;
use crate::files::settings::SaveFormat;
use crate::plugins::{FileView, FileViewEvent, Snapshot};

impl TextFile {
    fn replace_text(&mut self, target: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(edit) = edit_towards(self.editor.read(cx).text(), target) else {
            return false;
        };
        self.editor
            .update(cx, |editor, cx| apply_edits(editor, &[edit], window, cx))
    }
}

impl FileView for TextFile {
    fn reload(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let decoded = take_decoded(&self.path, bytes)
            .ok_or_else(|| anyhow!("it is not a UTF-8 text file"))?;
        let changed = self.replace_text(&decoded.text, window, cx);
        self.saved = decoded.fingerprint;
        self.line_ending = decoded.line_ending;
        self.byte_order_mark = decoded.text.starts_with(BYTE_ORDER_MARK);
        if !changed {
            cx.emit(FileViewEvent::Edited);
        }
        cx.emit(FileViewEvent::StatusChanged);
        Ok(())
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        !self.saved.matches(self.editor.read(cx).text())
    }

    fn snapshot(&mut self, cx: &mut Context<Self>) -> Option<Result<Snapshot>> {
        let text = self.editor.read(cx).text().clone();
        let line_ending = self.line_ending;
        let fingerprint = Arc::new(OnceLock::new());
        let encode = {
            let fingerprint = fingerprint.clone();
            move || {
                fingerprint.set(Fingerprint::of(&text)).ok();
                line_ending.encode(String::from(&text))
            }
        };
        let file = cx.weak_entity();
        let snapshot = Snapshot::new(encode).on_saved(move |cx| {
            if let Some(saved) = fingerprint.get().copied() {
                file.update(cx, |file, _| file.saved = saved).ok();
            }
        });
        Some(Ok(snapshot))
    }

    fn prepare_save(&mut self, format: &SaveFormat, window: &mut Window, cx: &mut Context<Self>) {
        let line_ending = format_for_save(&self.editor, format, self.line_ending, window, cx);
        if line_ending != self.line_ending {
            self.line_ending = line_ending;
            cx.emit(FileViewEvent::StatusChanged);
        }
    }

    fn unsaved_text(&self, cx: &App) -> Option<Rope> {
        self.text(cx)
    }

    fn restore_unsaved(&mut self, text: Rope, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text(&String::from(&text), window, cx);
    }

    fn set_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.path = path.to_path_buf();
        self.update_language(path, cx);
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        self.render_status_items(cx)
    }

    fn go_to(&mut self, line: usize, column: usize, window: &mut Window, cx: &mut Context<Self>) {
        let position = Position::new(
            u32::try_from(line).unwrap_or(u32::MAX),
            u32::try_from(column).unwrap_or(u32::MAX),
        );
        self.editor.update(cx, |editor, cx| {
            editor.set_cursor_position(position, window, cx)
        });
    }

    fn text(&self, cx: &App) -> Option<Rope> {
        Some(self.editor.read(cx).text().clone())
    }

    fn set_gutter_markers(&mut self, markers: Vec<GutterMarker>, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_gutter_markers(markers, cx));
    }
}
