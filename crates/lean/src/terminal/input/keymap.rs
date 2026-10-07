use gpui_kit::{App, KeyBinding, NoAction, actions};

actions!(terminal, [Paste]);

pub(crate) const KEY_CONTEXT: &str = "Terminal";

const SHELL_KEYSTROKES: [&str; 7] = [
    "ctrl-c",
    "ctrl-w",
    "ctrl-b",
    "ctrl-s",
    "ctrl-o",
    "tab",
    "shift-tab",
];

pub(crate) fn bind_keys(cx: &mut App) {
    let shell_bindings = SHELL_KEYSTROKES
        .into_iter()
        .map(|keystroke| KeyBinding::new(keystroke, NoAction, Some(KEY_CONTEXT)));
    let paste_bindings = [
        KeyBinding::new("ctrl-shift-v", Paste, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-insert", Paste, Some(KEY_CONTEXT)),
    ];
    cx.bind_keys(shell_bindings.chain(paste_bindings));
}
