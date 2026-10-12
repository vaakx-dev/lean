use std::ops::Range;

use gpui_kit::component::input::{EditorState, Rope};
use gpui_kit::*;

pub(in crate::editor) type TextEdit = (Range<usize>, String);

pub(in crate::editor) fn apply_edits(
    editor: &mut EditorState,
    edits: &[TextEdit],
    window: &mut Window,
    cx: &mut Context<EditorState>,
) -> bool {
    if edits.is_empty() {
        return false;
    }
    let selection = editor.selected_range();
    let scroll = editor.scroll_offset();
    editor.replace_text_in_ranges(edits, window, cx);
    let selection = map_offset(edits, selection.start)..map_offset(edits, selection.end);
    editor.set_selected_range(selection, cx);
    editor.set_scroll_offset(scroll, cx);
    true
}

pub(super) fn edit_towards(text: &Rope, target: &str) -> Option<TextEdit> {
    let prefix = common_prefix(text, target);
    let limit = text.len().min(target.len()) - prefix;
    let suffix = common_suffix(text, target, limit);
    let removed = prefix..text.len() - suffix;
    let inserted = &target[prefix..target.len() - suffix];
    (!removed.is_empty() || !inserted.is_empty()).then(|| (removed, inserted.to_owned()))
}

fn map_offset(edits: &[TextEdit], offset: usize) -> usize {
    let mut shift = 0isize;
    for (range, text) in edits {
        if offset <= range.start {
            break;
        }
        let start = range.start.saturating_add_signed(shift);
        if offset < range.end {
            return start + (offset - range.start).min(text.len());
        }
        shift += text.len() as isize - range.len() as isize;
    }
    offset.saturating_add_signed(shift)
}

fn common_prefix(text: &Rope, target: &str) -> usize {
    let target_bytes = target.as_bytes();
    let mut len = 0;
    for chunk in text.chunks() {
        let same = matching_len(chunk.bytes(), target_bytes[len..].iter().copied());
        len += same;
        if same < chunk.len() {
            break;
        }
    }
    floor_boundary(target, len)
}

fn common_suffix(text: &Rope, target: &str, limit: usize) -> usize {
    let target_bytes = target.as_bytes();
    let mut len = 0;
    for chunk in text.chunks_at(text.len()).0.reversed() {
        let rest = &target_bytes[..target_bytes.len() - len];
        let same = matching_len(chunk.bytes().rev(), rest.iter().rev().copied());
        len += same;
        if same < chunk.len() || len >= limit {
            break;
        }
    }
    let mut len = len.min(limit);
    while !target.is_char_boundary(target.len() - len) {
        len -= 1;
    }
    len
}

fn matching_len(left: impl Iterator<Item = u8>, right: impl Iterator<Item = u8>) -> usize {
    left.zip(right)
        .take_while(|(left, right)| left == right)
        .count()
}

fn floor_boundary(text: &str, mut index: usize) -> usize {
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
