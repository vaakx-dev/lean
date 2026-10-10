use gpui_kit::component::Theme;
use gpui_kit::{Font, Hsla, Pixels, font};

use super::palette::Palette;
use crate::terminal::font::TerminalFont;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorAppearance {
    Solid,
    Hollow,
    Hidden,
}

pub(crate) struct TerminalAppearance {
    pub(crate) palette: Palette,
    pub(crate) font: Font,
    pub(crate) font_size: Pixels,
    pub(crate) line_height: f32,
    pub(crate) cursor: CursorAppearance,
}

impl TerminalAppearance {
    pub(crate) fn new(
        theme: &Theme,
        terminal_font: TerminalFont,
        cursor: CursorAppearance,
    ) -> Self {
        Self {
            palette: Palette::new(theme),
            font: font(terminal_font.family),
            font_size: terminal_font.size,
            line_height: terminal_font.line_height,
            cursor,
        }
    }

    pub(crate) fn background(&self) -> Hsla {
        self.palette.background()
    }
}
