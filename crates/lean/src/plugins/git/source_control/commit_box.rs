use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::Textarea;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, v_flex};
use gpui_kit::*;

use super::SourceControl;
use super::keymap::{COMMIT_MESSAGE_CONTEXT, Commit};

impl SourceControl {
    pub(super) fn render_commit_box(&self, cx: &Context<Self>) -> Div {
        v_flex()
            .flex_shrink_0()
            .px_2()
            .pb_2()
            .gap_2()
            .child(
                div()
                    .key_context(COMMIT_MESSAGE_CONTEXT)
                    .on_action(cx.listener(Self::commit))
                    .child(Textarea::new(&self.commit_message).small()),
            )
            .child(
                Button::new("commit")
                    .primary()
                    .small()
                    .w_full()
                    .icon(Icon::empty().path("icons/check.svg"))
                    .label("Commit")
                    .loading(self.committing)
                    .disabled(self.committing)
                    .on_click(cx.listener(|source_control, _, window, cx| {
                        source_control.commit(&Commit, window, cx)
                    })),
            )
    }
}
