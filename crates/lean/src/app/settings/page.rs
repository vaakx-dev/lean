use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::{
    CONFIRM_QUIT_RUNNING_WORK, CONFIRM_UNSAVED_CLOSE, LOG_LEVEL, LogLevel, RESTORE_WINDOW_BOUNDS,
    STARTUP_OPEN, StartupFolder, TITLE_BAR, TitleBarStyle,
};
use crate::recent_projects::settings::{MENU_LIMIT, RECENT_LIMIT};
use crate::settings::ui::fields::{Number, group, number, select, switch};
use crate::workspace::settings::RESTORE_SESSION;

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("General")
        .icon(Icon::empty().path("icons/settings-2.svg"))
        .description("How Lean starts, its window, and what it confirms before closing.")
        .group(startup_group())
        .group(window_group())
        .group(confirmations_group())
        .group(recent_projects_group())
        .group(advanced_group())
}

fn startup_group() -> SettingGroup {
    group(
        "Startup",
        [
            select(
                &STARTUP_OPEN,
                "On startup open",
                "Folder opened when Lean starts without one.",
                [
                    (StartupFolder::Cwd, "Current folder"),
                    (StartupFolder::LastProject, "Last project"),
                ],
            )
            .keywords(["launch", "working directory", "recent"]),
            switch(
                &RESTORE_SESSION,
                "Restore open editors",
                "Reopen the tabs of each project when it opens.",
            )
            .keywords(["session", "tabs", "reopen"]),
        ],
    )
}

fn window_group() -> SettingGroup {
    group(
        "Window",
        [
            select(
                &TITLE_BAR,
                "Title bar",
                "Use Lean's own title bar or the system one.",
                [
                    (TitleBarStyle::Lean, "Lean"),
                    (TitleBarStyle::System, "System"),
                ],
            )
            .requires_restart()
            .keywords(["decorations", "titlebar", "window frame"]),
            switch(
                &RESTORE_WINDOW_BOUNDS,
                "Restore window position and size",
                "Reopen at the last size and position.",
            )
            .keywords(["bounds", "geometry", "maximized"]),
        ],
    )
}

fn confirmations_group() -> SettingGroup {
    group(
        "Confirmations",
        [
            switch(
                &CONFIRM_UNSAVED_CLOSE,
                "Confirm closing unsaved files",
                "Ask before closing an editor or window with unsaved changes.",
            )
            .keywords(["prompt", "dirty", "discard"]),
            switch(
                &CONFIRM_QUIT_RUNNING_WORK,
                "Warn before quitting with running work",
                "Warn about running terminals and kernels.",
            )
            .keywords(["prompt", "processes", "exit"]),
        ],
    )
}

fn recent_projects_group() -> SettingGroup {
    group(
        "Recent projects",
        [
            number(
                &RECENT_LIMIT,
                "Recent projects to remember",
                "Projects kept in the list, 1–100.",
                Number::range(1., 100.),
            )
            .keywords(["history", "folders"]),
            number(
                &MENU_LIMIT,
                "Shown in File › Open Recent",
                "The rest are listed under More….",
                Number::range(1., 30.),
            )
            .keywords(["menu", "history"]),
        ],
    )
}

fn advanced_group() -> SettingGroup {
    group(
        "Advanced",
        [select(
            &LOG_LEVEL,
            "Log level",
            "How much Lean writes to its log. The RUST_LOG variable overrides it.",
            [
                (LogLevel::Error, "Error"),
                (LogLevel::Warn, "Warning"),
                (LogLevel::Info, "Info"),
                (LogLevel::Debug, "Debug"),
                (LogLevel::Trace, "Trace"),
            ],
        )
        .keywords(["logging", "verbose", "debug", "RUST_LOG"])],
    )
}
