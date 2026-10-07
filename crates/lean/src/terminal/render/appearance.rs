use gpui_kit::component::Theme;
use gpui_kit::{Font, Hsla, Pixels, font};

use super::palette::Palette;

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
    pub(crate) cursor: CursorAppearance,
}

impl TerminalAppearance {
    pub(crate) fn new(theme: &Theme, cursor: CursorAppearance) -> Self {
        Self {
            palette: Palette::new(theme),
            font: font(theme.mono_font_family.clone()),
            font_size: theme.mono_font_size,
            cursor,
        }
    }

    pub(crate) fn background(&self) -> Hsla {
        self.palette.background()
    }
}
