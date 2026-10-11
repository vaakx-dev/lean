mod options;
mod status;
mod viewer;

use std::path::Path;

use anyhow::Result;
use gpui_kit::component::input::{Editor, EditorState, GutterMarker, InputEvent, Position, Rope};
use gpui_kit::*;

use super::file_text::{LineEnding, decode_text, same_text};
use super::minimap::{Minimap, render_minimap};
use super::saving::format_for_save;
use super::settings::{MINIMAP_ENABLED, font_family, font_size, line_height};
use crate::files::Language;
use crate::files::settings::SaveFormat;
use crate::plugins::{FileView, FileViewEvent, Snapshot};
use crate::settings::Settings;
use crate::toolchain::Toolchains;
use options::EditorOptions;
use viewer::language_for;

pub(crate) use viewer::TEXT_VIEWER;

const BYTE_ORDER_MARK: char = '\u{feff}';

pub(super) struct TextFile {
    editor: Entity<EditorState>,
    minimap: Option<Entity<Minimap>>,
    toolchains: Entity<Toolchains>,
    saved: Rope,
    cursor: Position,
    language: Language,
    line_ending: LineEnding,
    byte_order_mark: bool,
    options: EditorOptions,
    _subscriptions: [Subscription; 3],
}

impl EventEmitter<FileViewEvent> for TextFile {}

impl TextFile {
    fn new(
        path: &Path,
        content: String,
        line_ending: LineEnding,
        toolchains: Entity<Toolchains>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let language = language_for(path, content.len());
        let options = EditorOptions::read(cx);
        let byte_order_mark = content.starts_with(BYTE_ORDER_MARK);
        let editor = cx.new(|cx| {
            let state = EditorState::new(window, cx)
                .language(language.id())
                .searchable(true);
            options.configure(state).default_value(content)
        });
        let minimap = MINIMAP_ENABLED
            .get(cx)
            .then(|| cx.new(|cx| Minimap::new(editor.clone(), cx)));
        let saved = editor.read(cx).text().clone();
        let cursor = editor.read(cx).cursor_position();
        let subscriptions = [
            cx.subscribe(&editor, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.emit(FileViewEvent::Edited);
                }
            }),
            cx.observe(&editor, |text, editor, cx| text.sync_cursor(editor, cx)),
            cx.observe_global_in::<Settings>(window, |text, window, cx| {
                text.apply_settings(window, cx)
            }),
        ];
        Self {
            editor,
            minimap,
            toolchains,
            saved,
            cursor,
            language,
            line_ending,
            byte_order_mark,
            options,
            _subscriptions: subscriptions,
        }
    }

    fn sync_cursor(&mut self, editor: Entity<EditorState>, cx: &mut Context<Self>) {
        let cursor = editor.read(cx).cursor_position();
        if self.cursor != cursor {
            self.cursor = cursor;
            cx.emit(FileViewEvent::StatusChanged);
        }
    }

    fn apply_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let options = EditorOptions::read(cx);
        if options != self.options {
            let previous = std::mem::replace(&mut self.options, options);
            self.editor.update(cx, |editor, cx| {
                options.apply(&previous, editor, window, cx)
            });
            cx.emit(FileViewEvent::StatusChanged);
        }
        self.sync_minimap(cx);
        cx.notify();
    }

    fn sync_minimap(&mut self, cx: &mut Context<Self>) {
        let enabled = MINIMAP_ENABLED.get(cx);
        if enabled == self.minimap.is_some() {
            return;
        }
        self.minimap = enabled.then(|| {
            let editor = self.editor.clone();
            cx.new(|cx| Minimap::new(editor, cx))
        });
    }
}

impl FileView for TextFile {
    fn reload(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let (text, line_ending) = decode_text(bytes)?;
        let byte_order_mark = text.starts_with(BYTE_ORDER_MARK);
        let changed = self.editor.update(cx, |editor, cx| {
            if *editor.text() == text.as_str() {
                return false;
            }
            let selection = editor.selected_range();
            let scroll = editor.scroll_offset();
            editor.replace_all(text, window, cx);
            editor.set_selected_range(selection, cx);
            editor.set_scroll_offset(scroll, cx);
            true
        });
        self.saved = self.editor.read(cx).text().clone();
        self.line_ending = line_ending;
        self.byte_order_mark = byte_order_mark;
        if !changed {
            cx.emit(FileViewEvent::Edited);
        }
        cx.emit(FileViewEvent::StatusChanged);
        Ok(())
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        !same_text(self.editor.read(cx).text(), &self.saved)
    }

    fn snapshot(&mut self, cx: &mut Context<Self>) -> Option<Result<Snapshot>> {
        let text = self.editor.read(cx).text().clone();
        let saved = text.clone();
        let line_ending = self.line_ending;
        let file = cx.weak_entity();
        let snapshot =
            Snapshot::new(move || line_ending.encode(String::from(&text))).on_saved(move |cx| {
                file.update(cx, |file, _| file.saved = saved).ok();
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
        Some(self.editor.read(cx).text().clone())
    }

    fn restore_unsaved(&mut self, text: Rope, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.replace_all(String::from(&text), window, cx)
        });
    }

    fn set_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        let language = language_for(path, self.editor.read(cx).text().len());
        if language.id() != self.language.id() {
            self.editor
                .update(cx, |editor, cx| editor.set_highlighter(language.id(), cx));
            self.language = language;
            cx.emit(FileViewEvent::StatusChanged);
        }
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

impl Focusable for TextFile {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl Render for TextFile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editor = Editor::new(&self.editor)
            .bordered(false)
            .p_0()
            .h(relative(1.))
            .font_family(font_family(cx))
            .text_size(font_size(cx))
            .line_height(relative(line_height(cx)));
        div()
            .flex()
            .size_full()
            .child(div().flex_1().min_w_0().h_full().child(editor))
            .children(
                self.minimap
                    .as_ref()
                    .map(|minimap| render_minimap(minimap, cx)),
            )
    }
}
