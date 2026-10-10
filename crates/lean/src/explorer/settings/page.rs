use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;

use super::{
    AUTO_REVEAL, COMPACT_FOLDERS, CONFIRM_DELETE, DELETE_TO, DeleteTo, FILE_ICONS, FOLDERS_FIRST,
    FileIcons, GIT_IGNORED, GitIgnored, INDENT, NATURAL_SORT, OPEN_ON, OpenOn, ROW_HEIGHT,
    SHOW_DOT_GIT, SHOW_HIDDEN,
};
use crate::project::EXCLUDE;
use crate::settings::ui::fields::{Number, group, list, number, select, switch};
use crate::watcher::DEBOUNCE_MS;

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Explorer")
        .icon(Icon::empty().path("icons/folder-tree.svg"))
        .description("What the file tree shows and how it behaves.")
        .group(group(
            "Visibility",
            [
                switch(
                    &SHOW_HIDDEN,
                    "Show hidden files",
                    "Show files and folders whose names start with a dot.",
                )
                .keywords(["dotfiles", "hidden"]),
                switch(
                    &SHOW_DOT_GIT,
                    "Show .git folder",
                    "Show the repository's own .git folder.",
                )
                .keywords(["repository"]),
                select(
                    &GIT_IGNORED,
                    "Git-ignored files",
                    "How files matched by .gitignore appear in the tree.",
                    [
                        (GitIgnored::Hide, "Hide"),
                        (GitIgnored::Dim, "Dim"),
                        (GitIgnored::Show, "Show"),
                    ],
                )
                .keywords(["gitignore", "ignored"]),
                list(
                    &EXCLUDE,
                    "Excluded paths",
                    "Comma-separated .gitignore-style patterns. Matches are hidden from the tree and never watched.",
                    "node_modules, target, .venv",
                )
                .stacked()
                .keywords(["exclude", "glob", "pattern", "node_modules"]),
            ],
        ))
        .group(group(
            "Sorting",
            [
                switch(
                    &FOLDERS_FIRST,
                    "Folders first",
                    "List folders before files.",
                )
                .keywords(["sort", "order", "directories"]),
                switch(
                    &NATURAL_SORT,
                    "Natural sort",
                    "Order numbers by value, so file2 comes before file10.",
                )
                .keywords(["sort", "order", "numeric"]),
            ],
        ))
        .group(group(
            "Behaviour",
            [
                select(
                    &OPEN_ON,
                    "Open files with",
                    "How many clicks it takes to open a file from the tree.",
                    [
                        (OpenOn::SingleClick, "Single click"),
                        (OpenOn::DoubleClick, "Double click"),
                    ],
                )
                .keywords(["click", "open mode"]),
                switch(
                    &AUTO_REVEAL,
                    "Reveal the active file",
                    "Expand the tree to the file in the active editor and select it.",
                )
                .keywords(["auto reveal", "follow", "sync", "track"]),
                switch(
                    &COMPACT_FOLDERS,
                    "Compact folders",
                    "Show a chain of folders that each hold one folder on a single row, like src/main/java.",
                )
                .keywords(["nested", "single child"]),
                switch(
                    &CONFIRM_DELETE,
                    "Confirm before deleting",
                    "Ask before deleting a file or folder.",
                )
                .keywords(["delete", "prompt", "warning"]),
                select(
                    &DELETE_TO,
                    "Deleting",
                    "What happens to deleted files and folders.",
                    [
                        (DeleteTo::Trash, "Move to Trash"),
                        (DeleteTo::Permanent, "Delete permanently"),
                    ],
                )
                .keywords(["delete", "trash", "recycle bin", "remove"]),
            ],
        ))
        .group(group(
            "Look",
            [
                select(
                    &FILE_ICONS,
                    "File icons",
                    "Icons shown before file names.",
                    [
                        (FileIcons::Coloured, "Coloured"),
                        (FileIcons::Monochrome, "Monochrome"),
                        (FileIcons::None, "None"),
                    ],
                )
                .keywords(["icon theme", "colour", "color"]),
                number(
                    &ROW_HEIGHT,
                    "Row height",
                    "Height of each row in the tree.",
                    Number::range(20., 36.).unit("px"),
                )
                .keywords(["density", "compact", "spacing"]),
                number(
                    &INDENT,
                    "Indent",
                    "Extra indent for each nested level.",
                    Number::range(4., 24.).unit("px"),
                )
                .keywords(["indentation", "nesting", "tree"]),
            ],
        ))
        .group(group(
            "File watching",
            [number(
                &DEBOUNCE_MS,
                "File change debounce",
                "How long to wait for more changes on disk before refreshing the tree and open files.",
                Number::range(50., 2000.).step(10.).unit("ms"),
            )
            .keywords(["watcher", "delay", "refresh", "inotify"])],
        ))
}
