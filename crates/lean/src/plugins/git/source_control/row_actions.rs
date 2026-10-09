use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex};
use gpui_kit::*;

pub(super) const ROW_GROUP: &str = "source-control-row";

pub(super) fn icon_button(id: &'static str, icon: &'static str, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
}

pub(super) fn hover_actions(actions: Vec<Button>, busy: bool) -> Stateful<Div> {
    h_flex()
        .id("row-actions")
        .flex_shrink_0()
        .gap(px(2.))
        .invisible()
        .group_hover(ROW_GROUP, |style| style.visible())
        .on_click(|_, _, cx| cx.stop_propagation())
        .children(actions.into_iter().map(|action| action.disabled(busy)))
}
