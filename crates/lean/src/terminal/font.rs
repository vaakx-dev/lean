use gpui_kit::{App, Pixels, SharedString, px};

use super::settings::{FONT_FAMILY, FONT_SIZE, LINE_HEIGHT};
use crate::editor::settings::{font_family as editor_font_family, font_size as editor_font_size};
use crate::theme::installed_font_names;

const MIN_FONT_SIZE: f32 = 6.;
const MAX_FONT_SIZE: f32 = 72.;
const MIN_LINE_HEIGHT: f32 = 1.;
const MAX_LINE_HEIGHT: f32 = 2.;

pub(crate) struct TerminalFont {
    pub(crate) family: SharedString,
    pub(crate) size: Pixels,
    pub(crate) line_height: f32,
}

impl TerminalFont {
    pub(crate) fn resolve(cx: &mut App) -> Self {
        Self {
            family: family(cx),
            size: size(cx),
            line_height: LINE_HEIGHT.get(cx).clamp(MIN_LINE_HEIGHT, MAX_LINE_HEIGHT),
        }
    }
}

fn family(cx: &mut App) -> SharedString {
    match FONT_FAMILY.get(cx) {
        Some(name) if is_installed(&name, cx) => name.into(),
        _ => editor_font_family(cx),
    }
}

fn size(cx: &App) -> Pixels {
    match FONT_SIZE.get(cx) {
        Some(size) => px(size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE)),
        None => editor_font_size(cx),
    }
}

fn is_installed(name: &str, cx: &mut App) -> bool {
    installed_font_names(cx).iter().any(|font| font == name)
}
