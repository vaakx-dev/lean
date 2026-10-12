use gpui_kit::component::input::{EditorState, Rope, RopeExt as _};
use gpui_kit::*;

use crate::editor::file_text::LineEnding;
use crate::editor::text_file::{TextEdit, apply_edits};
use crate::files::settings::SaveFormat;

pub(in crate::editor) fn format_for_save(
    editor: &Entity<EditorState>,
    format: &SaveFormat,
    line_ending: LineEnding,
    window: &mut Window,
    cx: &mut App,
) -> LineEnding {
    if format.changes_text() {
        let edits = format_edits(editor.read(cx).text(), format);
        if !edits.is_empty() {
            editor.update(cx, |editor, cx| apply_edits(editor, &edits, window, cx));
        }
    }
    line_ending.for_save(format.line_endings)
}

fn format_edits(text: &Rope, format: &SaveFormat) -> Vec<TextEdit> {
    let (mut edits, last_line_len) = if format.trim_trailing_whitespace {
        trailing_whitespace(text)
    } else {
        (Vec::new(), text.line_len(text.lines_len() - 1))
    };
    if format.final_newline && last_line_len > 0 {
        edits.push((text.len()..text.len(), "\n".to_owned()));
    }
    edits
}

fn trailing_whitespace(text: &Rope) -> (Vec<TextEdit>, usize) {
    let mut edits = Vec::new();
    let mut line_start = 0;
    let mut last_line_len = 0;
    for line in text.iter_lines() {
        let line_end = line_start + line.len();
        let trailing: usize = line
            .chars_at(line.len())
            .reversed()
            .take_while(|character| character.is_whitespace())
            .map(char::len_utf8)
            .sum();
        if trailing > 0 {
            edits.push((line_end - trailing..line_end, String::new()));
        }
        last_line_len = line.len() - trailing;
        line_start = line_end + 1;
    }
    (edits, last_line_len)
}
