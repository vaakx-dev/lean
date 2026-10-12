use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Font, Hsla, Pixels, SharedString, font, px};

use super::palette::Palette;
use crate::editor::settings::{font_family as editor_font_family, font_size as editor_font_size};
use crate::terminal::settings::{
    FONT_FAMILY, FONT_SIZE, LINE_HEIGHT, MAX_FONT_SIZE, MAX_LINE_HEIGHT, MIN_FONT_SIZE,
    MIN_LINE_HEIGHT,
};
use crate::theme::installed_font;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorAppearance {
    Solid,
    Hollow,
    Hidden,
}

#[derive(Clone)]
pub(crate) struct TerminalAppearance {
    pub(crate) palette: Palette,
    pub(crate) font: Font,
    pub(crate) font_size: Pixels,
    pub(crate) line_height: f32,
    pub(crate) cursor: CursorAppearance,
}

impl TerminalAppearance {
    pub(crate) fn resolve(cx: &mut App) -> Self {
        Self {
            font: font(font_family(cx)),
            font_size: font_size(cx),
            line_height: LINE_HEIGHT.get(cx).clamp(MIN_LINE_HEIGHT, MAX_LINE_HEIGHT),
            palette: Palette::new(cx.theme()),
            cursor: CursorAppearance::Hidden,
        }
    }

    pub(crate) fn with_cursor(&self, cursor: CursorAppearance) -> Self {
        Self {
            cursor,
            ..self.clone()
        }
    }

    pub(crate) fn background(&self) -> Hsla {
        self.palette.background()
    }
}

fn font_family(cx: &mut App) -> SharedString {
    FONT_FAMILY
        .get(cx)
        .and_then(|name| installed_font(&name, cx))
        .unwrap_or_else(|| editor_font_family(cx))
}

fn font_size(cx: &App) -> Pixels {
    match FONT_SIZE.get(cx) {
        Some(size) => px(size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE)),
        None => editor_font_size(cx),
    }
}
