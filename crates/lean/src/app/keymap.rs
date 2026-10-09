use gpui_kit::{App, KeyBinding};

use super::actions::{
    CloseActiveEditor, NextEditor, OpenFolder, OpenRecent, PreviousEditor, Quit, SaveActiveEditor,
    ShowPanel, ToggleSidebar, ToggleTerminal,
};
use crate::{explorer, terminal};

pub(super) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("ctrl-o", OpenFolder, None),
        KeyBinding::new("ctrl-r", OpenRecent, None),
        KeyBinding::new("ctrl-s", SaveActiveEditor, None),
        KeyBinding::new("ctrl-w", CloseActiveEditor, None),
        KeyBinding::new("ctrl-pagedown", NextEditor, None),
        KeyBinding::new("ctrl-pageup", PreviousEditor, None),
        KeyBinding::new("ctrl-tab", NextEditor, None),
        KeyBinding::new("ctrl-shift-tab", PreviousEditor, None),
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-`", ToggleTerminal, None),
        KeyBinding::new(
            "ctrl-shift-e",
            ShowPanel {
                id: explorer::PANEL.id,
            },
            None,
        ),
    ]);
    explorer::bind_keys(cx);
    terminal::bind_keys(cx);
}
