use gpui_kit::{App, KeyBinding, actions};

use crate::app::keymap;

actions!(terminal, [CopySelection, Paste]);

pub(crate) const KEY_CONTEXT: &str = "Terminal";

pub(crate) fn bind_keys(cx: &mut App) {
    keymap::bind_defaults(
        cx,
        [
            KeyBinding::new("ctrl-shift-c", CopySelection, Some(KEY_CONTEXT)),
            KeyBinding::new("ctrl-insert", CopySelection, Some(KEY_CONTEXT)),
            KeyBinding::new("ctrl-shift-v", Paste, Some(KEY_CONTEXT)),
            KeyBinding::new("shift-insert", Paste, Some(KEY_CONTEXT)),
        ],
    );
}
