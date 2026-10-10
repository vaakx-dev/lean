use gpui_kit::component::list::ListItem;
use gpui_kit::component::{Icon, Theme, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

const ROW_HEIGHT: Pixels = px(34.);
const MARKER_SIZE: Pixels = px(6.);

pub(super) fn heading(title: &'static str, theme: &Theme) -> Div {
    div()
        .px_2()
        .py_1p5()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.muted_foreground)
        .child(title)
}

pub(super) fn setting_row(
    label: &'static str,
    modified: bool,
    control: impl IntoElement,
    theme: &Theme,
) -> Div {
    h_flex()
        .relative()
        .h(ROW_HEIGHT)
        .justify_between()
        .gap_2p5()
        .pl_3()
        .pr_2()
        .text_sm()
        .when(modified, |row| row.child(modified_marker(theme)))
        .child(label)
        .child(control)
}

pub(super) fn link_row(
    id: &'static str,
    icon: &'static str,
    label: &'static str,
    theme: &Theme,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> ListItem {
    ListItem::new(id)
        .h(ROW_HEIGHT)
        .px_2()
        .rounded(theme.radius)
        .text_sm()
        .child(
            h_flex()
                .gap_2p5()
                .child(
                    Icon::empty()
                        .path(icon)
                        .size_3p5()
                        .text_color(theme.muted_foreground),
                )
                .child(label),
        )
        .on_click(move |_, window, cx| on_click(window, cx))
}

fn modified_marker(theme: &Theme) -> Div {
    div()
        .absolute()
        .left(px(3.))
        .top_0()
        .bottom_0()
        .flex()
        .items_center()
        .child(div().size(MARKER_SIZE).rounded_full().bg(theme.primary))
}
