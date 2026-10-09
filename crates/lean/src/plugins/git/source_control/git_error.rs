use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, Theme, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SourceControl;
use crate::plugins::git::repo::GitReadError;

impl SourceControl {
    pub(super) fn render_git_error(&self, error: GitReadError, cx: &Context<Self>) -> AnyElement {
        let theme = cx.theme();
        v_flex()
            .id("git-error")
            .size_full()
            .overflow_y_scroll()
            .p_3()
            .gap_3()
            .text_sm()
            .child(
                h_flex()
                    .gap_2()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(
                        Icon::empty()
                            .path("icons/triangle-alert.svg")
                            .size_4()
                            .flex_shrink_0()
                            .text_color(theme.warning),
                    )
                    .child("Git could not read this repository"),
            )
            .child(
                div()
                    .text_color(theme.muted_foreground)
                    .child(error.message),
            )
            .when_some(error.trust_command, |panel, command| {
                panel.child(trust_hint(command, theme))
            })
            .child(
                h_flex().child(
                    Button::new("retry-git")
                        .small()
                        .outline()
                        .icon(Icon::empty().path("icons/refresh-cw.svg"))
                        .label("Try Again")
                        .on_click(
                            cx.listener(|source_control, _, _, cx| source_control.refresh(cx)),
                        ),
                ),
            )
            .into_any_element()
    }
}

fn trust_hint(command: SharedString, theme: &Theme) -> Div {
    let copied_command = command.clone();
    v_flex()
        .gap_2()
        .child(
            div()
                .text_color(theme.muted_foreground)
                .child("If you trust this folder, mark it as safe in your git config:"),
        )
        .child(
            div()
                .p_2()
                .rounded(theme.radius)
                .bg(theme.muted)
                .font_family(theme.mono_font_family.clone())
                .text_xs()
                .child(command),
        )
        .child(
            h_flex().child(
                Button::new("copy-trust-command")
                    .small()
                    .ghost()
                    .icon(Icon::empty().path("icons/copy.svg"))
                    .label("Copy Command")
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(copied_command.to_string()))
                    }),
            ),
        )
}
