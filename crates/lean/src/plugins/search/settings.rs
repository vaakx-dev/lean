mod page;

use crate::settings::Setting;

pub(crate) use page::settings_page;

pub(crate) const MATCH_CASE: Setting<bool> = Setting::new("search.match_case", false);
pub(crate) const WHOLE_WORD: Setting<bool> = Setting::new("search.whole_word", false);
pub(crate) const REGEX: Setting<bool> = Setting::new("search.regex", false);
pub(crate) const SMART_CASE: Setting<bool> = Setting::new("search.smart_case", false);
pub(crate) const SHOW_FILTERS: Setting<bool> = Setting::new("search.show_filters", false);

pub(crate) const INCLUDE_HIDDEN: Setting<bool> = Setting::new("search.include_hidden", false);
pub(crate) const RESPECT_GITIGNORE: Setting<bool> = Setting::new("search.respect_gitignore", true);
pub(crate) const FOLLOW_SYMLINKS: Setting<bool> = Setting::new("search.follow_symlinks", false);
pub(crate) const EXCLUDE: Setting<Vec<String>> = Setting::new("search.exclude", Vec::new());
pub(crate) const MAX_FILE_SIZE_MB: Setting<u32> = Setting::new("search.max_file_size_mb", 4);

pub(crate) const MAX_RESULTS: Setting<usize> = Setting::new("search.max_results", 2000);
pub(crate) const TYPING_DELAY_MS: Setting<u64> = Setting::new("search.typing_delay_ms", 150);

pub(crate) const MAX_FILE_SIZE_RANGE: (u32, u32) = (1, 64);
pub(crate) const MAX_RESULTS_RANGE: (usize, usize) = (100, 100_000);
pub(crate) const TYPING_DELAY_RANGE: (u64, u64) = (0, 1000);
