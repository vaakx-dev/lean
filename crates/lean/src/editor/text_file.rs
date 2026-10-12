mod decoded;
mod edits;
mod file_view;
mod look;
mod options;
mod status;
mod viewer;

use std::path::{Path, PathBuf};

use gpui_kit::component::Theme;
use gpui_kit::component::input::{Editor, EditorState, InputEvent, Position};
use gpui_kit::*;

use super::file_text::{Fingerprint, LineEnding};
use super::minimap::{Minimap, render_minimap};
use super::settings::MINIMAP_ENABLED;
use crate::files::Language;
use crate::plugins::FileViewEvent;
use crate::settings::Settings;
use crate::toolchain::Toolchains;
use decoded::DecodedText;
use look::EditorLook;
use options::EditorOptions;
use viewer::language_for;

pub(super) use decoded::discard_decoded;
pub(super) use edits::{TextEdit, apply_edits};
pub(crate) use viewer::TEXT_VIEWER;

const BYTE_ORDER_MARK: char = '\u{feff}';

pub(super) struct TextFile {
    editor: Entity<EditorState>,
    minimap: Option<Entity<Minimap>>,
    toolchains: Entity<Toolchains>,
    path: PathBuf,
    saved: Fingerprint,
    cursor: Position,
    language: Language,
    line_ending: LineEnding,
    byte_order_mark: bool,
    options: EditorOptions,
    look: EditorLook,
    _subscriptions: [Subscription; 4],
}

impl EventEmitter<FileViewEvent> for TextFile {}

impl TextFile {
    fn new(
        path: PathBuf,
        decoded: DecodedText,
        toolchains: Entity<Toolchains>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let language = language_for(&path, decoded.text.len());
        let options = EditorOptions::read(cx);
        let byte_order_mark = decoded.text.starts_with(BYTE_ORDER_MARK);
        let editor = cx.new(|cx| {
            let state = EditorState::new(window, cx)
                .language(language.id())
                .searchable(true);
            options.configure(state).default_value(decoded.text)
        });
        let minimap = MINIMAP_ENABLED
            .get(cx)
            .then(|| cx.new(|cx| Minimap::new(editor.clone(), cx)));
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
            cx.observe_global::<Theme>(|text, cx| text.sync_look(false, cx)),
        ];
        Self {
            editor,
            minimap,
            toolchains,
            path,
            saved: decoded.fingerprint,
            cursor,
            language,
            line_ending: decoded.line_ending,
            byte_order_mark,
            options,
            look: EditorLook::read(cx),
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
        let minimap_toggled = self.sync_minimap(cx);
        self.sync_look(minimap_toggled, cx);
    }

    fn sync_look(&mut self, force: bool, cx: &mut Context<Self>) {
        let look = EditorLook::read(cx);
        if force || look != self.look {
            self.look = look;
            cx.notify();
        }
    }

    fn sync_minimap(&mut self, cx: &mut Context<Self>) -> bool {
        let enabled = MINIMAP_ENABLED.get(cx);
        if enabled == self.minimap.is_some() {
            return false;
        }
        self.minimap = enabled.then(|| {
            let editor = self.editor.clone();
            cx.new(|cx| Minimap::new(editor, cx))
        });
        true
    }

    fn update_language(&mut self, path: &Path, cx: &mut Context<Self>) {
        let language = language_for(path, self.editor.read(cx).text().len());
        if language.id() != self.language.id() {
            self.editor
                .update(cx, |editor, cx| editor.set_highlighter(language.id(), cx));
            self.language = language;
            cx.emit(FileViewEvent::StatusChanged);
        }
    }
}

impl Focusable for TextFile {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl Render for TextFile {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let editor = Editor::new(&self.editor)
            .bordered(false)
            .p_0()
            .h(relative(1.))
            .font_family(self.look.font_family.clone())
            .text_size(self.look.font_size)
            .line_height(relative(self.look.line_height));
        div()
            .flex()
            .size_full()
            .child(div().flex_1().min_w_0().h_full().child(editor))
            .children(
                self.minimap
                    .as_ref()
                    .map(|minimap| render_minimap(minimap, self.look.minimap_width)),
            )
    }
}
