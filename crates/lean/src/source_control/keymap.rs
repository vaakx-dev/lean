use gpui_kit::{App, KeyBinding, actions};

actions!(source_control, [Commit]);

pub(super) const COMMIT_MESSAGE_CONTEXT: &str = "CommitMessage";

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-enter",
        Commit,
        Some("CommitMessage > Input"),
    )]);
}
