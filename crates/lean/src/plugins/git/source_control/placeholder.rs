use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{AnyElement, App, IntoElement, ParentElement, Styled, div};

pub(super) fn placeholder(message: &'static str, cx: &App) -> AnyElement {
    div()
        .size_full()
        .p_4()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(message)
        .into_any_element()
}
