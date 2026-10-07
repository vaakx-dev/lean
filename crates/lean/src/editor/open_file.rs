use std::path::PathBuf;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{Editor, EditorState, InputEvent, Position, TabSize};
use gpui_kit::*;

use super::minimap::Minimap;
use super::{EditorArea, EditorAreaEvent};
use crate::files::{Language, file_name};

const HIGHLIGHT_SIZE_LIMIT: usize = 4 * 1024 * 1024;
const TAB_SIZE: TabSize = TabSize {
    tab_size: 4,
    hard_tabs: false,
};

pub(crate) struct OpenFile {
    pub(super) path: PathBuf,
    pub(super) name: SharedString,
    pub(super) editor: Entity<EditorState>,
    pub(super) minimap: Entity<Minimap>,
    pub(super) dirty: bool,
    cursor: Position,
    language: Language,
    _editor_subscriptions: [Subscription; 2],
}

impl OpenFile {
    pub(super) fn new(
        path: PathBuf,
        content: String,
        window: &mut Window,
        cx: &mut Context<EditorArea>,
    ) -> Self {
        let language = if content.len() > HIGHLIGHT_SIZE_LIMIT {
            Language::PLAIN
        } else {
            Language::for_path(&path)
        };
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
        let cursor = editor.read(cx).cursor_position();
        let editor_subscriptions = [
            cx.subscribe(&editor, |area, editor, event: &InputEvent, cx| {
                area.handle_editor_event(editor, event, cx)
            }),
            cx.observe(&editor, |area, editor, cx| area.sync_cursor(editor, cx)),
        ];
        Self {
            name: file_name(&path),
            path,
            editor,
            minimap,
            dirty: false,
            cursor,
            language,
            _editor_subscriptions: editor_subscriptions,
        }
    }

    pub(crate) fn cursor_position(&self) -> Position {
        self.cursor
    }

    pub(crate) fn language_name(&self) -> &'static str {
        self.language.display_name()
    }

    pub(super) fn render_editor(&self, cx: &App) -> Editor {
        let theme = cx.theme();
        Editor::new(&self.editor)
            .bordered(false)
            .p_0()
            .h(relative(1.))
            .font_family(theme.mono_font_family.clone())
            .text_size(theme.mono_font_size)
    }
}

impl EditorArea {
    fn handle_editor_event(
        &mut self,
        editor: Entity<EditorState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change)
            && let Some(file) = self.files.iter_mut().find(|file| file.editor == editor)
            && !file.dirty
        {
            file.dirty = true;
            cx.notify();
        }
        cx.emit(EditorAreaEvent::ActiveEditorChanged);
    }

    fn sync_cursor(&mut self, editor: Entity<EditorState>, cx: &mut Context<Self>) {
        let cursor = editor.read(cx).cursor_position();
        if let Some(file) = self.files.iter_mut().find(|file| file.editor == editor)
            && file.cursor != cursor
        {
            file.cursor = cursor;
            cx.emit(EditorAreaEvent::ActiveEditorChanged);
        }
    }
}
