mod page;

use serde::{Deserialize, Serialize};

use crate::settings::Setting;

pub(crate) use page::settings_page;

pub(crate) const SHOW_HIDDEN: Setting<bool> = Setting::new("explorer.show_hidden", true);
pub(crate) const SHOW_DOT_GIT: Setting<bool> = Setting::new("explorer.show_dot_git", false);
pub(crate) const GIT_IGNORED: Setting<GitIgnored> =
    Setting::new("explorer.git_ignored", GitIgnored::Hide);
pub(crate) const FOLDERS_FIRST: Setting<bool> = Setting::new("explorer.folders_first", true);
pub(crate) const NATURAL_SORT: Setting<bool> = Setting::new("explorer.natural_sort", false);
pub(crate) const OPEN_ON: Setting<OpenOn> = Setting::new("explorer.open_on", OpenOn::SingleClick);
pub(crate) const AUTO_REVEAL: Setting<bool> = Setting::new("explorer.auto_reveal", false);
pub(crate) const COMPACT_FOLDERS: Setting<bool> = Setting::new("explorer.compact_folders", false);
pub(crate) const CONFIRM_DELETE: Setting<bool> = Setting::new("explorer.confirm_delete", true);
pub(crate) const DELETE_TO: Setting<DeleteTo> = Setting::new("explorer.delete_to", DeleteTo::Trash);
pub(crate) const FILE_ICONS: Setting<FileIcons> =
    Setting::new("explorer.file_icons", FileIcons::Coloured);
pub(crate) const ROW_HEIGHT: Setting<u32> = Setting::new("explorer.row_height", 26);
pub(crate) const INDENT: Setting<u32> = Setting::new("explorer.indent", 10);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GitIgnored {
    Hide,
    Dim,
    Show,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpenOn {
    SingleClick,
    DoubleClick,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DeleteTo {
    Trash,
    Permanent,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FileIcons {
    Coloured,
    Monochrome,
    None,
}
