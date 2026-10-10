use gpui_kit::{App, actions};

use crate::editor::settings::FONT_SIZE;

actions!(editor, [IncreaseFontSize, DecreaseFontSize, ResetFontSize]);

const STEP: f32 = 1.;
const MIN_SIZE: f32 = 8.;
const MAX_SIZE: f32 = 32.;

pub(super) fn init(cx: &mut App) {
    cx.on_action(|_: &IncreaseFontSize, cx| change_font_size(STEP, cx));
    cx.on_action(|_: &DecreaseFontSize, cx| change_font_size(-STEP, cx));
    cx.on_action(|_: &ResetFontSize, cx| FONT_SIZE.reset(cx));
}

fn change_font_size(delta: f32, cx: &mut App) {
    let size = (FONT_SIZE.get(cx).round() + delta).clamp(MIN_SIZE, MAX_SIZE);
    FONT_SIZE.set(size, cx);
}
