use gpui_kit::App;

use super::config::SearchConfig;
use super::settings::{MATCH_CASE, REGEX, WHOLE_WORD};

#[derive(Clone, Copy, Default)]
pub(super) struct SearchOptions {
    pub(super) match_case: bool,
    pub(super) whole_word: bool,
    pub(super) regex: bool,
}

impl SearchOptions {
    pub(super) fn from_settings(cx: &App) -> Self {
        Self {
            match_case: MATCH_CASE.get(cx),
            whole_word: WHOLE_WORD.get(cx),
            regex: REGEX.get(cx),
        }
    }
}

pub(super) struct SearchQuery {
    pub(super) pattern: String,
    pub(super) options: SearchOptions,
    pub(super) include: String,
    pub(super) exclude: String,
    pub(super) config: SearchConfig,
}
