use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::{
    COMPACT_FOLDERS, CONFIRM_DISCARD, DIFF_CONTEXT_LINES, EXPLORER_DECORATIONS,
    EXPLORER_FOLDER_DOTS, ExplorerDecorations, GUTTER_BASE, GUTTER_ENABLED, HISTORY_LIMIT,
    INCLUDE_UNTRACKED, MAX_DIFF_CONTEXT_LINES, MAX_HISTORY_LIMIT, MIN_HISTORY_LIMIT,
    NOTHING_STAGED, NothingStaged, SHOW_HISTORY, STATUS_BAR_BRANCH, VIEW_MODE, ViewMode,
};
use crate::plugins::git::diff::DiffBase;
use crate::settings::ui::fields::{Number, group, number, select, switch};

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Source Control")
        .icon(Icon::empty().path("icons/git-branch.svg"))
        .description("Changes, committing and change indicators.")
        .group(changes_group())
        .group(committing_group())
        .group(indicators_group())
}

fn changes_group() -> SettingGroup {
    group(
        "Changes",
        [
            select(
                &VIEW_MODE,
                "View mode",
                "Show changed files as a folder tree or a flat list.",
                [(ViewMode::Tree, "Tree"), (ViewMode::List, "List")],
            )
            .keywords(["layout", "flat", "folders"]),
            switch(
                &COMPACT_FOLDERS,
                "Compact folders",
                "Join folders that contain a single folder into one row.",
            )
            .keywords(["collapse", "nested", "tree"])
            .enabled_when(|cx| VIEW_MODE.get(cx) == ViewMode::Tree),
            switch(
                &INCLUDE_UNTRACKED,
                "Include untracked files",
                "List new files that are not tracked by git yet.",
            )
            .keywords(["new files", "status"]),
            switch(
                &SHOW_HISTORY,
                "Show history",
                "List recent commits below the changes.",
            )
            .keywords(["commits", "log", "graph"]),
            number(
                &HISTORY_LIMIT,
                "History length",
                "Commits loaded at a time. Load more from the end of the list.",
                Number::range(MIN_HISTORY_LIMIT as f64, MAX_HISTORY_LIMIT as f64)
                    .step(10.)
                    .unit("commits"),
            )
            .keywords(["commits", "log", "limit", "load more"])
            .enabled_when(|cx| SHOW_HISTORY.get(cx)),
        ],
    )
}

fn committing_group() -> SettingGroup {
    group(
        "Committing",
        [
            select(
                &NOTHING_STAGED,
                "When nothing is staged",
                "What committing does when no changes are staged.",
                [
                    (NothingStaged::Ask, "Ask"),
                    (NothingStaged::StageAll, "Stage all and commit"),
                    (NothingStaged::DoNothing, "Do nothing"),
                ],
            )
            .keywords(["commit", "stage all", "smart commit"]),
            switch(
                &CONFIRM_DISCARD,
                "Confirm discard",
                "Ask before discarding changes or deleting untracked files.",
            )
            .keywords(["revert", "delete", "undo", "warning"]),
        ],
    )
}

fn indicators_group() -> SettingGroup {
    group(
        "Indicators",
        [
            switch(
                &GUTTER_ENABLED,
                "Change markers",
                "Mark added, changed and deleted lines next to the line numbers.",
            )
            .keywords(["gutter", "diff", "lines"]),
            select(
                &GUTTER_BASE,
                "Compare markers with",
                "The version that change markers are measured against.",
                [
                    (DiffBase::Index, "Staged version (index)"),
                    (DiffBase::Staged, "Last commit (HEAD)"),
                ],
            )
            .keywords(["gutter", "diff", "index", "head"])
            .enabled_when(|cx| GUTTER_ENABLED.get(cx)),
            select(
                &EXPLORER_DECORATIONS,
                "Explorer file status",
                "How the explorer marks changed files.",
                [
                    (ExplorerDecorations::Letter, "Letter"),
                    (ExplorerDecorations::Dot, "Dot"),
                    (ExplorerDecorations::Off, "Off"),
                ],
            )
            .keywords(["decorations", "badge", "file tree", "modified"]),
            switch(
                &EXPLORER_FOLDER_DOTS,
                "Explorer folder dots",
                "Mark folders that contain changes.",
            )
            .keywords(["decorations", "file tree", "folders"]),
            switch(
                &STATUS_BAR_BRANCH,
                "Branch in status bar",
                "Show the current branch in the status bar.",
            )
            .keywords(["branch", "status bar", "footer"]),
            number(
                &DIFF_CONTEXT_LINES,
                "Diff context lines",
                "Unchanged lines shown around each change in diff tabs.",
                Number::range(0., MAX_DIFF_CONTEXT_LINES as f64).unit("lines"),
            )
            .keywords(["diff", "context", "hunk"]),
        ],
    )
}
