use gpui_kit::component::{Icon, Theme, h_flex};
use gpui_kit::*;

use super::chevron_path;

pub(crate) fn list_row(id: impl Into<ElementId>, height: Pixels, theme: &Theme) -> Stateful<Div> {
    h_flex()
        .id(id)
        .w_full()
        .h(height)
        .flex_shrink_0()
        .gap_1p5()
        .pr_1()
        .overflow_hidden()
        .rounded(theme.radius)
}

pub(crate) fn clickable_list_row(
    id: impl Into<ElementId>,
    height: Pixels,
    theme: &Theme,
) -> Stateful<Div> {
    list_row(id, height, theme)
        .cursor_pointer()
        .hover(|style| style.bg(theme.list_hover))
}

pub(crate) fn chevron(expanded: bool, theme: &Theme) -> Icon {
    Icon::empty()
        .path(chevron_path(expanded))
        .size_4()
        .flex_shrink_0()
        .text_color(theme.muted_foreground)
}

pub(crate) fn count_badge(count: usize, theme: &Theme) -> Div {
    div()
        .flex_shrink_0()
        .min_w(px(18.))
        .px_1p5()
        .flex()
        .justify_center()
        .rounded_full()
        .bg(theme.accent)
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(count.to_string())
}
