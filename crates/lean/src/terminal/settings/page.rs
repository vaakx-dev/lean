use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::environment_field::environment_field;
use super::{
    BELL, BellMode, CONFIRM_CLOSE_RUNNING, COPY_ON_SELECT, CURSOR_BLINK, CURSOR_SHAPE,
    ClipboardAccess, CursorShapeChoice, FONT_FAMILY, FONT_SIZE, LINE_HEIGHT, MAX_FONT_SIZE,
    MAX_LINE_HEIGHT, MIN_FONT_SIZE, MIN_LINE_HEIGHT, OSC52, PANEL_HEIGHT, SCROLLBACK, SHELL_ARGS,
    SHELL_LOGIN, SHELL_PROGRAM, WORKING_DIRECTORY, WorkingDirectory, default_shell_program,
};
use crate::editor::settings::font_size as editor_font_size;
use crate::settings::ui::fields::{
    Number, PathKind, font, group, list, number, path, select, switch,
};

pub(crate) fn settings_page(cx: &App) -> SettingPage {
    SettingPage::new("Terminal")
        .icon(Icon::empty().path("icons/square-terminal.svg"))
        .description("Shell, appearance and behaviour of the integrated terminal.")
        .group(shell_group())
        .group(appearance_group(cx))
        .group(behaviour_group())
        .group(panel_group())
}

fn shell_group() -> SettingGroup {
    group(
        "Shell",
        [
            path(
                &SHELL_PROGRAM,
                "Shell",
                "Program started in new terminals. Empty uses $SHELL.",
                PathKind::File,
                default_shell_program(),
            )
            .keywords(["program", "bash", "zsh", "fish", "powershell"]),
            list(
                &SHELL_ARGS,
                "Shell arguments",
                "Comma-separated arguments passed to the shell.",
                "--norc",
            )
            .keywords(["args", "flags"]),
            switch(
                &SHELL_LOGIN,
                "Login shell",
                "Start the shell as a login shell so it reads your profile.",
            )
            .keywords(["profile", "-l", "--login"]),
            environment_field(),
            select(
                &WORKING_DIRECTORY,
                "Start in",
                "Folder new terminals open in.",
                [
                    (WorkingDirectory::ProjectRoot, "Project root"),
                    (WorkingDirectory::ActiveFileFolder, "Active file's folder"),
                    (WorkingDirectory::Home, "Home folder"),
                ],
            )
            .keywords(["working directory", "cwd", "folder"]),
        ],
    )
}

fn appearance_group(cx: &App) -> SettingGroup {
    let editor_size = editor_font_size(cx).as_f32().to_string();
    group(
        "Appearance",
        [
            font(
                &FONT_FAMILY,
                "Font",
                "Empty uses the editor font.",
                "Same as the editor",
            )
            .keywords(["typeface", "family"]),
            number(
                &FONT_SIZE,
                "Font size",
                "Empty uses the editor font size.",
                Number::range(MIN_FONT_SIZE.into(), MAX_FONT_SIZE.into())
                    .unit("px")
                    .placeholder(editor_size),
            ),
            number(
                &LINE_HEIGHT,
                "Line height",
                "Multiple of the font size.",
                Number::range(MIN_LINE_HEIGHT.into(), MAX_LINE_HEIGHT.into())
                    .step(0.1)
                    .unit("×"),
            )
            .keywords(["spacing"]),
            select(
                &CURSOR_SHAPE,
                "Cursor shape",
                "Programs running in the terminal can change it.",
                [
                    (CursorShapeChoice::Block, "Block"),
                    (CursorShapeChoice::Beam, "Beam"),
                    (CursorShapeChoice::Underline, "Underline"),
                ],
            )
            .keywords(["caret"]),
            switch(&CURSOR_BLINK, "Blinking cursor", "").keywords(["caret", "blink"]),
        ],
    )
}

fn behaviour_group() -> SettingGroup {
    group(
        "Behaviour",
        [
            number(
                &SCROLLBACK,
                "Scrollback",
                "Lines kept above the visible screen.",
                Number::range(0., 100_000.).step(1000.).unit("lines"),
            )
            .keywords(["history", "buffer"]),
            switch(
                &COPY_ON_SELECT,
                "Copy on select",
                "Copy selected text to the clipboard as soon as you select it.",
            )
            .keywords(["clipboard", "selection"]),
            select(
                &BELL,
                "Bell",
                "What happens when a program rings the bell.",
                [
                    (BellMode::None, "Nothing"),
                    (BellMode::Visual, "Flash the terminal"),
                    (BellMode::System, "System sound"),
                ],
            )
            .keywords(["beep", "alert", "sound"]),
            select(
                &OSC52,
                "Clipboard access",
                "Let programs use the clipboard through OSC 52 escape codes, for example over SSH.",
                [
                    (ClipboardAccess::Off, "Off"),
                    (ClipboardAccess::Copy, "Copy only"),
                    (ClipboardAccess::CopyPaste, "Copy and paste"),
                ],
            )
            .keywords(["osc52", "osc 52", "clipboard", "ssh", "tmux"]),
            switch(
                &CONFIRM_CLOSE_RUNNING,
                "Confirm closing busy terminals",
                "Ask before closing a terminal that is still running a command.",
            )
            .keywords(["kill", "close", "running", "process"]),
        ],
    )
}

fn panel_group() -> SettingGroup {
    group(
        "Panel",
        [number(
            &PANEL_HEIGHT,
            "Panel height",
            "You can also drag the top edge of the terminal panel.",
            Number::range(120., 800.).step(10.).unit("px"),
        )
        .keywords(["size", "resize", "dock"])],
    )
}
