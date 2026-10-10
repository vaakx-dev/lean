use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::{
    EXCLUDE, FOLLOW_SYMLINKS, INCLUDE_HIDDEN, MATCH_CASE, MAX_FILE_SIZE_MB, MAX_FILE_SIZE_RANGE,
    MAX_RESULTS, MAX_RESULTS_RANGE, REGEX, RESPECT_GITIGNORE, SHOW_FILTERS, SMART_CASE,
    TYPING_DELAY_MS, TYPING_DELAY_RANGE, WHOLE_WORD,
};
use crate::settings::ui::fields::{Number, group, list, number, switch};

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Search")
        .icon(Icon::empty().path("icons/search.svg"))
        .description("Defaults, scope and limits for finding text in files.")
        .group(defaults_group())
        .group(scope_group())
        .group(results_group())
}

fn defaults_group() -> SettingGroup {
    group(
        "Defaults",
        [
            switch(
                &MATCH_CASE,
                "Match case",
                "Start new searches with case-sensitive matching.",
            )
            .keywords(["case sensitive", "uppercase", "lowercase"]),
            switch(
                &WHOLE_WORD,
                "Match whole word",
                "Start new searches matching whole words only.",
            )
            .keywords(["word boundary", "whole word"]),
            switch(
                &REGEX,
                "Use regular expression",
                "Start new searches treating the query as a regular expression.",
            )
            .keywords(["regex", "pattern"]),
            switch(
                &SMART_CASE,
                "Smart case",
                "Match case only when the query contains an uppercase letter.",
            )
            .keywords(["case sensitive", "case insensitive", "uppercase"]),
            switch(
                &SHOW_FILTERS,
                "Show file filters",
                "Show the files to include and exclude fields by default.",
            )
            .keywords(["include", "exclude", "details", "glob"]),
        ],
    )
    .description("Toggles in the Search panel start from these values.")
}

fn scope_group() -> SettingGroup {
    group(
        "Scope",
        [
            switch(
                &INCLUDE_HIDDEN,
                "Include hidden files",
                "Search files and folders whose names start with a dot.",
            )
            .keywords(["dotfiles", "hidden"]),
            switch(
                &RESPECT_GITIGNORE,
                "Respect .gitignore",
                "Skip files ignored by Git.",
            )
            .keywords(["ignore", "git", "exclude"]),
            switch(
                &FOLLOW_SYMLINKS,
                "Follow symbolic links",
                "Search inside linked files and folders.",
            )
            .keywords(["symlink", "link"]),
            list(
                &EXCLUDE,
                "Excluded paths",
                "Comma-separated globs that are never searched.",
                "node_modules, target, *.min.js",
            )
            .keywords(["exclude", "ignore", "glob", "skip"])
            .stacked(),
            number(
                &MAX_FILE_SIZE_MB,
                "Maximum file size",
                "Larger files are skipped.",
                Number::range(
                    f64::from(MAX_FILE_SIZE_RANGE.0),
                    f64::from(MAX_FILE_SIZE_RANGE.1),
                )
                .unit("MB"),
            )
            .keywords(["large files", "limit", "size"]),
        ],
    )
}

fn results_group() -> SettingGroup {
    group(
        "Results",
        [
            number(
                &MAX_RESULTS,
                "Maximum results",
                "Stop searching after this many matches.",
                Number::range(MAX_RESULTS_RANGE.0 as f64, MAX_RESULTS_RANGE.1 as f64).step(100.),
            )
            .keywords(["limit", "matches", "truncate"]),
            number(
                &TYPING_DELAY_MS,
                "Typing delay",
                "Wait this long after typing stops before searching.",
                Number::range(TYPING_DELAY_RANGE.0 as f64, TYPING_DELAY_RANGE.1 as f64)
                    .step(10.)
                    .unit("ms"),
            )
            .keywords(["debounce", "delay", "live search"]),
        ],
    )
}
