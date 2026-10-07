use gpui_kit::{App, KeyBinding};

use super::actions::{
    CloseActiveEditor, NextEditor, OpenFolder, PreviousEditor, Quit, SaveActiveEditor,
    ShowExplorer, ShowSearch, ShowSourceControl, ToggleSidebar, ToggleTerminal,
};
use crate::{explorer, source_control, terminal};

pub(super) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("ctrl-o", OpenFolder, None),
        KeyBinding::new("ctrl-s", SaveActiveEditor, None),
        KeyBinding::new("ctrl-w", CloseActiveEditor, None),
        KeyBinding::new("ctrl-pagedown", NextEditor, None),
        KeyBinding::new("ctrl-pageup", PreviousEditor, None),
        KeyBinding::new("ctrl-tab", NextEditor, None),
        KeyBinding::new("ctrl-shift-tab", PreviousEditor, None),
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-`", ToggleTerminal, None),
        KeyBinding::new("ctrl-shift-e", ShowExplorer, None),
        KeyBinding::new("ctrl-shift-f", ShowSearch, None),
        KeyBinding::new("ctrl-shift-g", ShowSourceControl, None),
    ]);
    explorer::bind_keys(cx);
    source_control::bind_keys(cx);
    terminal::bind_keys(cx);
}
