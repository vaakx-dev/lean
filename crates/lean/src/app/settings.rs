mod page;

use serde::{Deserialize, Serialize};

use crate::settings::Setting;

pub(crate) use page::settings_page;

pub(crate) const STARTUP_OPEN: Setting<StartupFolder> =
    Setting::new("startup.open", StartupFolder::Cwd);
pub(crate) const TITLE_BAR: Setting<TitleBarStyle> =
    Setting::new("window.decorations", TitleBarStyle::Lean);
pub(crate) const RESTORE_WINDOW_BOUNDS: Setting<bool> =
    Setting::new("window.restore_bounds", false);
pub(crate) const CONFIRM_UNSAVED_CLOSE: Setting<bool> =
    Setting::new("files.confirm_unsaved_close", true);
pub(crate) const CONFIRM_QUIT_RUNNING_WORK: Setting<bool> =
    Setting::new("confirm.quit_running_work", true);
pub(crate) const LOG_LEVEL: Setting<LogLevel> = Setting::new("log.level", LogLevel::Warn);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StartupFolder {
    Cwd,
    LastProject,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TitleBarStyle {
    Lean,
    System,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
