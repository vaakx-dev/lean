use gpui_kit::component::empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia};
use gpui_kit::component::{ActiveTheme as _, Icon};
use gpui_kit::*;

const ICON: &str = "icons/file-image.svg";
const ICON_SIZE: Pixels = px(48.);

pub(super) fn empty_state(message: impl Into<SharedString>, cx: &App) -> Empty {
    let icon = Icon::empty()
        .path(ICON)
        .size(ICON_SIZE)
        .text_color(cx.theme().muted_foreground);
    Empty::new().size_full().header(
        EmptyHeader::new()
            .media(EmptyMedia::new().child(icon))
            .description(EmptyDescription::new().child(message.into())),
    )
}
