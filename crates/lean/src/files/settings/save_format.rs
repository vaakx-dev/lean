use gpui_kit::App;

use super::{FINAL_NEWLINE, FinalNewline, LINE_ENDINGS, LineEndings, TRIM_TRAILING_WHITESPACE};

#[derive(Clone, Copy)]
pub(crate) struct SaveFormat {
    pub(crate) trim_trailing_whitespace: bool,
    pub(crate) final_newline: bool,
    pub(crate) line_endings: LineEndings,
}

impl SaveFormat {
    pub(crate) fn from_settings(cx: &App) -> Self {
        Self {
            trim_trailing_whitespace: TRIM_TRAILING_WHITESPACE.get(cx),
            final_newline: FINAL_NEWLINE.get(cx) == FinalNewline::Ensure,
            line_endings: LINE_ENDINGS.get(cx),
        }
    }

    pub(crate) fn without_text_changes(self) -> Self {
        Self {
            trim_trailing_whitespace: false,
            final_newline: false,
            ..self
        }
    }

    pub(crate) fn changes_text(&self) -> bool {
        self.trim_trailing_whitespace || self.final_newline
    }
}
