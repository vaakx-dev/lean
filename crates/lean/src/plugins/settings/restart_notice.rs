use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::setting::{SettingGroup, SettingItem};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex};
use gpui_kit::*;

use crate::app;

pub(super) fn restart_notice() -> SettingGroup {
    SettingGroup::new()
        .variant(GroupBoxVariant::Normal)
        .item(SettingItem::render(|_, _, cx| render_notice(cx)).keywords(["restart"]))
}

fn render_notice(cx: &App) -> Div {
    let theme = cx.theme();
    h_flex()
        .w_full()
        .gap_2()
        .px_3()
        .py_2()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.warning)
        .bg(theme.warning.opacity(0.1))
        .child(
            Icon::empty()
                .path("icons/triangle-alert.svg")
                .size_4()
                .text_color(theme.warning),
        )
        .child(
            div()
                .flex_1()
                .text_sm()
                .child("Restart Lean to apply plugin changes"),
        )
        .child(
            Button::new("restart-lean")
                .primary()
                .small()
                .label("Restart")
                .on_click(|_, _, cx| app::restart(cx)),
        )
}
