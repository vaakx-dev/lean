use gpui_kit::{App, KeyBinding, actions};

use super::PANEL;
use crate::app::actions::ShowPanel;
use crate::app::keymap;

actions!(source_control, [Commit]);

pub(super) const COMMIT_MESSAGE_CONTEXT: &str = "CommitMessage";

pub(crate) fn bind_keys(cx: &mut App) {
    keymap::bind_command(
        cx,
        "lean::ShowSourceControl",
        "Show Source Control",
        [KeyBinding::new(
            "ctrl-shift-g",
            ShowPanel { id: PANEL.id },
            None,
        )],
    );
    keymap::bind_defaults(
        cx,
        [KeyBinding::new(
            "ctrl-enter",
            Commit,
            Some("CommitMessage > Input"),
        )],
    );
}
