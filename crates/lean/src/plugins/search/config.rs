use std::time::Duration;

use gpui_kit::App;

use super::settings::{
    EXCLUDE, FOLLOW_SYMLINKS, INCLUDE_HIDDEN, MAX_FILE_SIZE_MB, MAX_FILE_SIZE_RANGE, MAX_RESULTS,
    MAX_RESULTS_RANGE, RESPECT_GITIGNORE, SMART_CASE, TYPING_DELAY_MS, TYPING_DELAY_RANGE,
};
use crate::files::settings::BYTES_PER_MEGABYTE;

#[derive(Clone, PartialEq)]
pub(super) struct SearchConfig {
    pub(super) smart_case: bool,
    pub(super) include_hidden: bool,
    pub(super) respect_gitignore: bool,
    pub(super) follow_symlinks: bool,
    pub(super) exclude: Vec<String>,
    pub(super) max_file_size: u64,
    pub(super) max_results: usize,
}

impl SearchConfig {
    pub(super) fn from_settings(cx: &App) -> Self {
        let (min_size, max_size) = MAX_FILE_SIZE_RANGE;
        let (min_results, max_results) = MAX_RESULTS_RANGE;
        Self {
            smart_case: SMART_CASE.get(cx),
            include_hidden: INCLUDE_HIDDEN.get(cx),
            respect_gitignore: RESPECT_GITIGNORE.get(cx),
            follow_symlinks: FOLLOW_SYMLINKS.get(cx),
            exclude: EXCLUDE.get(cx),
            max_file_size: u64::from(MAX_FILE_SIZE_MB.get(cx).clamp(min_size, max_size))
                * BYTES_PER_MEGABYTE,
            max_results: MAX_RESULTS.get(cx).clamp(min_results, max_results),
        }
    }
}

pub(super) fn typing_delay(cx: &App) -> Duration {
    let (min, max) = TYPING_DELAY_RANGE;
    Duration::from_millis(TYPING_DELAY_MS.get(cx).clamp(min, max))
}
