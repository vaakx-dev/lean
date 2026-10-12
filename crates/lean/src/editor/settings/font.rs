use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Pixels, SharedString, actions, px};

use super::{FONT_FAMILY, FONT_SIZE, LINE_HEIGHT};
use crate::theme::installed_font;

actions!(editor, [IncreaseFontSize, DecreaseFontSize, ResetFontSize]);

const FONT_SIZE_STEP: f32 = 1.;
const MIN_FONT_SIZE: f32 = 8.;
const MAX_FONT_SIZE: f32 = 32.;

pub(crate) fn init(cx: &mut App) {
    cx.on_action(|_: &IncreaseFontSize, cx| change_font_size(FONT_SIZE_STEP, cx));
    cx.on_action(|_: &DecreaseFontSize, cx| change_font_size(-FONT_SIZE_STEP, cx));
    cx.on_action(|_: &ResetFontSize, cx| FONT_SIZE.reset(cx));
}

pub(crate) fn font_family(cx: &mut App) -> SharedString {
    FONT_FAMILY
        .get(cx)
        .and_then(|name| installed_font(&name, cx))
        .unwrap_or_else(|| cx.theme().mono_font_family.clone())
}

pub(crate) fn font_size(cx: &App) -> Pixels {
    px(FONT_SIZE.get(cx).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE))
}

pub(crate) fn line_height(cx: &App) -> f32 {
    LINE_HEIGHT.get(cx).clamp(1., 2.5)
}

fn change_font_size(delta: f32, cx: &mut App) {
    let size = (FONT_SIZE.get(cx).round() + delta).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
    FONT_SIZE.set(size, cx);
}
