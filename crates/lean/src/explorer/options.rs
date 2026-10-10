use gpui_kit::{App, Context, Pixels, px};

use super::Explorer;
use super::settings::{
    AUTO_REVEAL, COMPACT_FOLDERS, FILE_ICONS, FOLDERS_FIRST, FileIcons, GIT_IGNORED, GitIgnored,
    INDENT, NATURAL_SORT, OPEN_ON, OpenOn, ROW_HEIGHT, SHOW_DOT_GIT, SHOW_HIDDEN,
};
use super::sorting::{Sorting, sort_entries};
use crate::project::DirectoryEntry;

const DOT_GIT: &str = ".git";

#[derive(Clone, PartialEq)]
pub(super) struct ExplorerOptions {
    pub(super) show_hidden: bool,
    pub(super) show_dot_git: bool,
    pub(super) git_ignored: GitIgnored,
    pub(super) sorting: Sorting,
    pub(super) open_on: OpenOn,
    pub(super) auto_reveal: bool,
    pub(super) compact_folders: bool,
    pub(super) file_icons: FileIcons,
    pub(super) row_height: Pixels,
    pub(super) indent: Pixels,
}

impl ExplorerOptions {
    pub(super) fn read(cx: &App) -> Self {
        Self {
            show_hidden: SHOW_HIDDEN.get(cx),
            show_dot_git: SHOW_DOT_GIT.get(cx),
            git_ignored: GIT_IGNORED.get(cx),
            sorting: Sorting {
                folders_first: FOLDERS_FIRST.get(cx),
                natural: NATURAL_SORT.get(cx),
            },
            open_on: OPEN_ON.get(cx),
            auto_reveal: AUTO_REVEAL.get(cx),
            compact_folders: COMPACT_FOLDERS.get(cx),
            file_icons: FILE_ICONS.get(cx),
            row_height: px(ROW_HEIGHT.get(cx).clamp(20, 36) as f32),
            indent: px(INDENT.get(cx).clamp(4, 24) as f32),
        }
    }

    pub(super) fn shows(&self, entry: &DirectoryEntry) -> bool {
        if entry.name == DOT_GIT {
            return self.show_dot_git;
        }
        if !self.show_hidden && entry.name.starts_with('.') {
            return false;
        }
        !entry.is_ignored || self.git_ignored != GitIgnored::Hide
    }

    pub(super) fn dims_ignored(&self) -> bool {
        self.git_ignored == GitIgnored::Dim
    }
}

impl Explorer {
    pub(super) fn apply_options(&mut self, cx: &mut Context<Self>) {
        let options = ExplorerOptions::read(cx);
        if options == self.options {
            return;
        }
        let resort = options.sorting != self.options.sorting;
        let starts_following = options.auto_reveal && !self.options.auto_reveal;
        self.options = options;
        if resort {
            let sorting = self.options.sorting;
            for entries in self.children.values_mut() {
                sort_entries(entries, sorting);
            }
        }
        self.rebuild_visible();
        cx.notify();
        if starts_following {
            self.followed = None;
            self.follow_active_editor(cx);
        }
    }
}
