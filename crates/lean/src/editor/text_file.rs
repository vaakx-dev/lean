mod language;
mod status;
mod viewer;

use std::path::Path;

use anyhow::Result;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{
    Editor, EditorState, GutterMarker, InputEvent, Position, Rope, TabSize,
};
use gpui_kit::*;

use super::file_text::{LineEnding, decode_text, same_text};
use super::minimap::{Minimap, render_minimap};
use crate::files::Language;
use crate::plugins::{FileView, FileViewEvent, Snapshot};
use crate::toolchain::Toolchains;
use language::language_for;

pub(crate) use viewer::TEXT_VIEWER;

const TAB_SIZE: TabSize = TabSize {
    tab_size: 4,
    hard_tabs: false,
};

pub(super) struct TextFile {
    editor: Entity<EditorState>,
    minimap: Entity<Minimap>,
    toolchains: Entity<Toolchains>,
    saved: Rope,
    cursor: Position,
    language: Language,
    line_ending: LineEnding,
    _subscriptions: [Subscription; 2],
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
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language(language.id())
                .line_number(true)
                .indent_guides(true)
                .folding(true)
                .searchable(true)
                .soft_wrap(false)
                .tab_size(TAB_SIZE)
                .default_value(content)
        });
        let minimap = cx.new(|cx| Minimap::new(editor.clone(), cx));
        let saved = editor.read(cx).text().clone();
        let cursor = editor.read(cx).cursor_position();
        let subscriptions = [
            cx.subscribe(&editor, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.emit(FileViewEvent::Edited);
                }
            }),
            cx.observe(&editor, |text, editor, cx| text.sync_cursor(editor, cx)),
        ];
        Self {
            editor,
            minimap,
            toolchains,
            saved,
            cursor,
            language,
            line_ending,
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
}

impl FileView for TextFile {
    fn reload(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let (text, line_ending) = decode_text(bytes)?;
        self.editor.update(cx, |editor, cx| {
            let selection = editor.selected_range();
            let scroll = editor.scroll_offset();
            editor.set_value(text, window, cx);
            editor.set_selected_range(selection, cx);
            editor.set_scroll_offset(scroll, cx);
            cx.emit(InputEvent::Change);
        });
        self.saved = self.editor.read(cx).text().clone();
        self.line_ending = line_ending;
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
                if let Err(error) = file.update(cx, |file, _| file.saved = saved) {
                    log::debug!("text file dropped after saving: {error:#}");
                }
            });
        Some(Ok(snapshot))
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
        let theme = cx.theme();
        div()
            .flex()
            .size_full()
            .child(
                div().flex_1().min_w_0().h_full().child(
                    Editor::new(&self.editor)
                        .bordered(false)
                        .p_0()
                        .h(relative(1.))
                        .font_family(theme.mono_font_family.clone())
                        .text_size(theme.mono_font_size),
                ),
            )
            .child(render_minimap(&self.minimap))
    }
}
