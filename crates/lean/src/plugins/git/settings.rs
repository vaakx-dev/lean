mod page;

use gpui_kit::App;
use serde::{Deserialize, Serialize};

use super::diff::DiffBase;
use crate::settings::Setting;

pub(crate) use page::settings_page;

pub(crate) const VIEW_MODE: Setting<ViewMode> = Setting::new("git.view_mode", ViewMode::Tree);
pub(crate) const COMPACT_FOLDERS: Setting<bool> = Setting::new("git.compact_folders", true);
pub(crate) const INCLUDE_UNTRACKED: Setting<bool> = Setting::new("git.include_untracked", true);
pub(crate) const SHOW_HISTORY: Setting<bool> = Setting::new("git.show_history", true);
pub(crate) const HISTORY_LIMIT: Setting<usize> = Setting::new("git.history_limit", 50);

pub(crate) const NOTHING_STAGED: Setting<NothingStaged> =
    Setting::new("git.commit_with_nothing_staged", NothingStaged::Ask);
pub(crate) const CONFIRM_DISCARD: Setting<bool> = Setting::new("git.confirm_discard", true);

pub(crate) const GUTTER_ENABLED: Setting<bool> = Setting::new("git.gutter.enabled", true);
pub(crate) const GUTTER_BASE: Setting<DiffBase> = Setting::new("git.gutter.base", DiffBase::Index);
pub(crate) const EXPLORER_DECORATIONS: Setting<ExplorerDecorations> =
    Setting::new("git.explorer_decorations", ExplorerDecorations::Letter);
pub(crate) const EXPLORER_FOLDER_DOTS: Setting<bool> =
    Setting::new("git.explorer_folder_dots", true);
pub(crate) const STATUS_BAR_BRANCH: Setting<bool> = Setting::new("git.status_bar_branch", true);
pub(crate) const DIFF_CONTEXT_LINES: Setting<u32> = Setting::new("git.diff_context_lines", 3);

pub(crate) const MIN_HISTORY_LIMIT: usize = 10;
pub(crate) const MAX_HISTORY_LIMIT: usize = 1000;
pub(crate) const MAX_DIFF_CONTEXT_LINES: u32 = 20;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ViewMode {
    Tree,
    List,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NothingStaged {
    Ask,
    StageAll,
    DoNothing,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExplorerDecorations {
    Letter,
    Dot,
    Off,
}

pub(crate) fn history_page_size(cx: &App) -> usize {
    HISTORY_LIMIT
        .get(cx)
        .clamp(MIN_HISTORY_LIMIT, MAX_HISTORY_LIMIT)
}

pub(crate) fn diff_context_lines(cx: &App) -> u32 {
    DIFF_CONTEXT_LINES.get(cx).min(MAX_DIFF_CONTEXT_LINES)
}
