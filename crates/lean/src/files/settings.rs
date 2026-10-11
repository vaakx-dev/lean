mod limits;
mod page;
mod save_format;

use serde::{Deserialize, Serialize};

use crate::settings::Setting;

pub(crate) use limits::{BYTES_PER_MEGABYTE, highlight_limit_bytes, init, max_file_bytes};
pub(crate) use page::settings_page;
pub(crate) use save_format::SaveFormat;

pub(crate) const AUTO_SAVE: Setting<AutoSave> = Setting::new("files.auto_save", AutoSave::Off);
pub(crate) const AUTO_SAVE_DELAY_MS: Setting<u64> = Setting::new("files.auto_save_delay_ms", 1000);
pub(crate) const TRIM_TRAILING_WHITESPACE: Setting<bool> =
    Setting::new("files.trim_trailing_whitespace", false);
pub(crate) const FINAL_NEWLINE: Setting<FinalNewline> =
    Setting::new("files.final_newline", FinalNewline::Leave);
pub(crate) const LINE_ENDINGS: Setting<LineEndings> =
    Setting::new("files.line_endings", LineEndings::Preserve);
pub(crate) const HOT_EXIT: Setting<bool> = Setting::new("files.hot_exit", false);
pub(crate) const EXTERNAL_CHANGES: Setting<ExternalChanges> =
    Setting::new("files.external_changes", ExternalChanges::Reload);
pub(crate) const EXTERNAL_CHANGE_NOTICE: Setting<bool> =
    Setting::new("files.external_change_notice", false);
pub(crate) const MAX_SIZE_MB: Setting<u32> = Setting::new("files.max_size_mb", DEFAULT_MAX_SIZE_MB);
pub(crate) const HIGHLIGHT_LIMIT_MB: Setting<u32> =
    Setting::new("files.highlight_limit_mb", DEFAULT_HIGHLIGHT_LIMIT_MB);

const DEFAULT_MAX_SIZE_MB: u32 = 256;
const DEFAULT_HIGHLIGHT_LIMIT_MB: u32 = 4;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AutoSave {
    Off,
    AfterDelay,
    OnFocusChange,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FinalNewline {
    Leave,
    Ensure,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LineEndings {
    Preserve,
    Lf,
    Crlf,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalChanges {
    Reload,
    Ask,
}
