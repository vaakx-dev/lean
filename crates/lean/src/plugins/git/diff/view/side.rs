use std::ops::Range;

use gpui_kit::component::highlighter::SyntaxHighlighter;
use gpui_kit::component::input::{HighlightStyleResolver, Rope, RopeExt as _};
use gpui_kit::{HighlightStyle, SharedString};

use crate::files::settings::highlight_limit_bytes;

const MAX_LINE_BYTES: usize = 4000;

pub(super) struct Side {
    text: Rope,
    highlighter: Option<SyntaxHighlighter>,
}

pub(super) struct SideLine {
    pub(super) text: SharedString,
    pub(super) styles: Vec<(Range<usize>, HighlightStyle)>,
}

impl Side {
    pub(super) fn empty() -> Self {
        Self {
            text: Rope::new(),
            highlighter: None,
        }
    }

    pub(super) fn new(text: &str, language: &str) -> Self {
        let text = Rope::from(text);
        let highlighter = (text.len() <= highlight_limit_bytes()).then(|| {
            let mut highlighter = SyntaxHighlighter::new(language);
            highlighter.update(None, &text, None);
            highlighter
        });
        Self { text, highlighter }
    }

    pub(super) fn line_count(&self) -> usize {
        self.text.lines_len()
    }

    pub(super) fn line(&self, line: usize, resolver: &dyn HighlightStyleResolver) -> SideLine {
        let full_line = self.text.slice_line(line);
        let shown_len = full_line.floor_char_boundary(full_line.len().min(MAX_LINE_BYTES));
        let text = String::from(full_line.slice(..shown_len));
        let start = self.text.line_start_offset(line);
        let end = start + text.len();
        let styles = match &self.highlighter {
            Some(highlighter) => highlighter
                .styles(&(start..end), resolver)
                .into_iter()
                .filter_map(|(range, style)| {
                    let range_start = range.start.max(start);
                    let range_end = range.end.min(end);
                    (range_start < range_end)
                        .then(|| (range_start - start..range_end - start, style))
                })
                .collect(),
            None => Vec::new(),
        };
        SideLine {
            text: text.into(),
            styles,
        }
    }
}
