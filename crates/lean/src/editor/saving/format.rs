use std::ops::Range;

use gpui_kit::component::input::EditorState;
use gpui_kit::*;

use crate::editor::file_text::LineEnding;
use crate::files::settings::SaveFormat;

struct FormattedText {
    text: String,
    removed: Vec<Range<usize>>,
}

impl FormattedText {
    fn map_offset(&self, offset: usize) -> usize {
        let removed: usize = self
            .removed
            .iter()
            .map(|range| range.end.min(offset).saturating_sub(range.start))
            .sum();
        offset - removed
    }
}

pub(in crate::editor) fn format_for_save(
    editor: &Entity<EditorState>,
    format: &SaveFormat,
    line_ending: LineEnding,
    window: &mut Window,
    cx: &mut App,
) -> LineEnding {
    if format.changes_text() {
        apply_text_format(editor, format, window, cx);
    }
    line_ending.for_save(format.line_endings)
}

fn apply_text_format(
    editor: &Entity<EditorState>,
    format: &SaveFormat,
    window: &mut Window,
    cx: &mut App,
) {
    let text = String::from(editor.read(cx).text());
    let Some(formatted) = format_text(&text, format) else {
        return;
    };
    editor.update(cx, |editor, cx| {
        let selection = editor.selected_range();
        let scroll = editor.scroll_offset();
        let selection = formatted.map_offset(selection.start)..formatted.map_offset(selection.end);
        editor.replace_all(formatted.text, window, cx);
        editor.set_selected_range(selection, cx);
        editor.set_scroll_offset(scroll, cx);
    });
}

fn format_text(text: &str, format: &SaveFormat) -> Option<FormattedText> {
    let mut output = String::with_capacity(text.len() + 1);
    let mut removed = Vec::new();
    let mut line_start = 0;
    for line in text.split_inclusive('\n') {
        let (content, newline) = match line.strip_suffix('\n') {
            Some(content) => (content, "\n"),
            None => (line, ""),
        };
        let kept = if format.trim_trailing_whitespace {
            content.trim_end()
        } else {
            content
        };
        if kept.len() < content.len() {
            removed.push(line_start + kept.len()..line_start + content.len());
        }
        output.push_str(kept);
        output.push_str(newline);
        line_start += line.len();
    }
    let adds_newline = format.final_newline && !output.is_empty() && !output.ends_with('\n');
    if adds_newline {
        output.push('\n');
    }
    (adds_newline || !removed.is_empty()).then_some(FormattedText {
        text: output,
        removed,
    })
}
