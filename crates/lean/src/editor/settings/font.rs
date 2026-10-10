use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Pixels, SharedString, px};

use super::{FONT_FAMILY, FONT_SIZE, LINE_HEIGHT};
use crate::theme::installed_font_names;

pub(crate) fn font_family(cx: &mut App) -> SharedString {
    FONT_FAMILY
        .get(cx)
        .filter(|name| is_installed(name, cx))
        .map(SharedString::from)
        .unwrap_or_else(|| cx.theme().mono_font_family.clone())
}

pub(crate) fn font_size(cx: &App) -> Pixels {
    px(FONT_SIZE.get(cx).clamp(8., 32.))
}

pub(crate) fn line_height(cx: &App) -> f32 {
    LINE_HEIGHT.get(cx).clamp(1., 2.5)
}

fn is_installed(name: &str, cx: &mut App) -> bool {
    installed_font_names(cx).iter().any(|font| font == name)
}
