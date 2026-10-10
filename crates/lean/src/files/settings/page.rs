use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::{
    AUTO_SAVE, AUTO_SAVE_DELAY_MS, AutoSave, EXTERNAL_CHANGE_NOTICE, EXTERNAL_CHANGES,
    ExternalChanges, FINAL_NEWLINE, FinalNewline, HIGHLIGHT_LIMIT_MB, HOT_EXIT, LINE_ENDINGS,
    LineEndings, MAX_SIZE_MB, TRIM_TRAILING_WHITESPACE,
};
use crate::settings::ui::fields::{Number, group, number, select, switch};

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Files")
        .icon(Icon::empty().path("icons/files.svg"))
        .description("Saving, external changes and file size limits.")
        .group(saving_group())
        .group(external_changes_group())
        .group(limits_group())
}

fn saving_group() -> SettingGroup {
    group(
        "Saving",
        [
            select(
                &AUTO_SAVE,
                "Auto save",
                "Save edited files automatically.",
                [
                    (AutoSave::Off, "Off"),
                    (AutoSave::AfterDelay, "After a delay"),
                    (AutoSave::OnFocusChange, "When focus changes"),
                ],
            )
            .keywords(["autosave", "save automatically"]),
            number(
                &AUTO_SAVE_DELAY_MS,
                "Auto save delay",
                "How long to wait after the last edit.",
                Number::range(200., 10_000.).step(100.).unit("ms"),
            )
            .keywords(["autosave", "debounce"])
            .enabled_when(|cx| AUTO_SAVE.get(cx) == AutoSave::AfterDelay),
            switch(
                &TRIM_TRAILING_WHITESPACE,
                "Trim trailing whitespace",
                "Remove spaces and tabs at the end of lines when saving.",
            )
            .keywords(["strip", "spaces", "format on save"]),
            select(
                &FINAL_NEWLINE,
                "Final newline",
                "Whether files end with a line break when saved.",
                [
                    (FinalNewline::Leave, "Leave as is"),
                    (FinalNewline::Ensure, "Add if missing"),
                ],
            )
            .keywords(["newline", "end of file", "eof"]),
            select(
                &LINE_ENDINGS,
                "Line endings on save",
                "Preserve keeps the file's most common line ending.",
                [
                    (LineEndings::Preserve, "Preserve"),
                    (LineEndings::Lf, "LF"),
                    (LineEndings::Crlf, "CRLF"),
                ],
            )
            .keywords(["eol", "crlf", "lf", "newline"]),
            switch(
                &HOT_EXIT,
                "Keep unsaved changes across restarts",
                "Back up unsaved files and restore them when the project opens again.",
            )
            .keywords(["hot exit", "backup", "restore", "unsaved"]),
        ],
    )
}

fn external_changes_group() -> SettingGroup {
    group(
        "External changes",
        [
            select(
                &EXTERNAL_CHANGES,
                "When a file changes on disk",
                "Files with unsaved changes always ask before saving over the disk version.",
                [
                    (ExternalChanges::Reload, "Reload it"),
                    (ExternalChanges::Ask, "Ask first"),
                ],
            )
            .keywords(["reload", "watch", "modified outside", "conflict"]),
            switch(
                &EXTERNAL_CHANGE_NOTICE,
                "Notify on reload",
                "Show a notification when a file is reloaded from disk.",
            )
            .keywords(["reload", "notification"])
            .enabled_when(|cx| EXTERNAL_CHANGES.get(cx) == ExternalChanges::Reload),
        ],
    )
}

fn limits_group() -> SettingGroup {
    group(
        "Limits",
        [
            number(
                &MAX_SIZE_MB,
                "Largest file to open",
                "Bigger files are not opened in the editor.",
                Number::range(1., 2048.).unit("MB"),
            )
            .keywords(["size", "large files"]),
            number(
                &HIGHLIGHT_LIMIT_MB,
                "Syntax highlighting limit",
                "Larger files open as plain text.",
                Number::range(1., 64.).unit("MB"),
            )
            .keywords(["size", "large files", "syntax", "colors"]),
        ],
    )
}
