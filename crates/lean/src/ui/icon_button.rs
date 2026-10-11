use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _};

pub(crate) fn icon_button(id: &'static str, icon: &'static str, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
}
