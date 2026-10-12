use std::mem;

use gpui_kit::SharedString;

use super::ansi::{incomplete_escape_start, strip_ansi};
use super::content::truncated;

pub(super) struct StreamText {
    max_lines: usize,
    plain: String,
    pending_escape: String,
    newlines: usize,
    trailing_newlines: usize,
    has_text: bool,
    display: SharedString,
}

impl StreamText {
    pub(super) fn new(text: &str, max_lines: usize) -> Self {
        let mut stream = Self {
            max_lines,
            plain: String::new(),
            pending_escape: String::new(),
            newlines: 0,
            trailing_newlines: 0,
            has_text: false,
            display: SharedString::default(),
        };
        stream.push(text);
        stream
    }

    pub(super) fn display(&self) -> SharedString {
        self.display.clone()
    }

    pub(super) fn push(&mut self, chunk: &str) {
        let mut raw = mem::take(&mut self.pending_escape);
        raw.push_str(chunk);
        if let Some(start) = incomplete_escape_start(&raw) {
            self.pending_escape = raw.split_off(start);
        }
        self.append(&strip_ansi(&raw));
        self.display = self.render();
    }

    fn append(&mut self, text: &str) {
        let room = self.max_lines.saturating_sub(self.newlines);
        if room > 0 {
            let end = text
                .match_indices('\n')
                .nth(room - 1)
                .map_or(text.len(), |(index, _)| index + 1);
            self.plain.push_str(&text[..end]);
        }
        self.newlines += text.matches('\n').count();
        let trimmed = text.trim_end_matches('\n');
        if trimmed.is_empty() {
            self.trailing_newlines += text.len();
        } else {
            self.has_text = true;
            self.trailing_newlines = text.len() - trimmed.len();
        }
    }

    fn line_count(&self) -> usize {
        if self.has_text {
            self.newlines - self.trailing_newlines + 1
        } else {
            0
        }
    }

    fn render(&self) -> SharedString {
        let line_count = self.line_count();
        if line_count <= self.max_lines {
            return self.plain.trim_end_matches('\n').to_string().into();
        }
        truncated(self.plain.lines(), line_count - self.max_lines)
    }
}
