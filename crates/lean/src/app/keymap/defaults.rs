use gpui_kit::{App, KeyBinding, actions};

use super::registry::{bind_command, bind_defaults};
use crate::app::actions::{
    CloseActiveEditor, NextEditor, OpenFolder, OpenRecent, OpenSettings, PreviousEditor, Quit,
    SaveActiveEditor, ShowPanel, ToggleSidebar, ToggleTerminal,
};
use crate::editor::settings::font::{DecreaseFontSize, IncreaseFontSize, ResetFontSize};
use crate::explorer;

actions!(editor, [GoToLine]);
actions!(explorer, [RevealActiveFile]);

pub(super) fn bind(cx: &mut App) {
    bind_defaults(
        cx,
        [
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("ctrl-o", OpenFolder, None),
            KeyBinding::new("ctrl-r", OpenRecent, None),
            KeyBinding::new("ctrl-,", OpenSettings, None),
            KeyBinding::new("ctrl-s", SaveActiveEditor, None),
            KeyBinding::new("ctrl-w", CloseActiveEditor, None),
            KeyBinding::new("ctrl-tab", NextEditor, None),
            KeyBinding::new("ctrl-pagedown", NextEditor, None),
            KeyBinding::new("ctrl-shift-tab", PreviousEditor, None),
            KeyBinding::new("ctrl-pageup", PreviousEditor, None),
            KeyBinding::new("ctrl-g", GoToLine, None),
            KeyBinding::new("ctrl-=", IncreaseFontSize, None),
            KeyBinding::new("ctrl-+", IncreaseFontSize, None),
            KeyBinding::new("ctrl--", DecreaseFontSize, None),
            KeyBinding::new("ctrl-0", ResetFontSize, None),
            KeyBinding::new("ctrl-b", ToggleSidebar, None),
            KeyBinding::new("ctrl-`", ToggleTerminal, None),
            KeyBinding::new("ctrl-alt-r", RevealActiveFile, None),
        ],
    );
    bind_command(
        cx,
        "lean::ShowExplorer",
        "Show Explorer",
        [KeyBinding::new(
            "ctrl-shift-e",
            ShowPanel {
                id: explorer::PANEL.id,
            },
            None,
        )],
    );
    explorer::bind_keys(cx);
    #[cfg(feature = "terminal")]
    crate::terminal::bind_keys(cx);
}
