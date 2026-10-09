use gpui_kit::component::Theme;
use gpui_kit::{Div, FontWeight, Hsla, ParentElement, Styled, div};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileStatus {
    Untracked,
    Added,
    Modified,
    Deleted,
    Renamed,
    Conflicted,
}

impl FileStatus {
    pub(crate) fn letter(self) -> &'static str {
        match self {
            FileStatus::Untracked => "U",
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Conflicted => "!",
        }
    }

    pub(crate) fn color(self, theme: &Theme) -> Hsla {
        match self {
            FileStatus::Untracked | FileStatus::Added => theme.green,
            FileStatus::Modified => theme.yellow,
            FileStatus::Deleted => theme.red,
            FileStatus::Renamed => theme.blue,
            FileStatus::Conflicted => theme.magenta,
        }
    }

    pub(crate) fn marker(self, theme: &Theme) -> Div {
        div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(self.color(theme))
            .child(self.letter())
    }

    pub(super) fn from_index(status: git2::Status) -> Option<Self> {
        if status.is_conflicted() {
            None
        } else if status.is_index_new() {
            Some(FileStatus::Added)
        } else if status.is_index_deleted() {
            Some(FileStatus::Deleted)
        } else if status.is_index_renamed() {
            Some(FileStatus::Renamed)
        } else if status.is_index_modified() || status.is_index_typechange() {
            Some(FileStatus::Modified)
        } else {
            None
        }
    }

    pub(super) fn from_worktree(status: git2::Status) -> Option<Self> {
        if status.is_conflicted() {
            Some(FileStatus::Conflicted)
        } else if status.is_wt_new() {
            Some(FileStatus::Untracked)
        } else if status.is_wt_deleted() {
            Some(FileStatus::Deleted)
        } else if status.is_wt_renamed() {
            Some(FileStatus::Renamed)
        } else if status.is_wt_modified() || status.is_wt_typechange() {
            Some(FileStatus::Modified)
        } else {
            None
        }
    }

    pub(super) fn from_git(status: git2::Status) -> Option<Self> {
        if status.is_conflicted() {
            Some(FileStatus::Conflicted)
        } else if status.is_wt_new() {
            Some(FileStatus::Untracked)
        } else if status.is_index_new() {
            Some(FileStatus::Added)
        } else if status.is_wt_deleted() || status.is_index_deleted() {
            Some(FileStatus::Deleted)
        } else if status.is_wt_renamed() || status.is_index_renamed() {
            Some(FileStatus::Renamed)
        } else if status.is_wt_modified()
            || status.is_index_modified()
            || status.is_wt_typechange()
            || status.is_index_typechange()
        {
            Some(FileStatus::Modified)
        } else {
            None
        }
    }
}
