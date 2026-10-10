use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::color::Colors;
use gpui_kit::{Font, Hsla};

use super::palette::{DIM_OPACITY, Palette};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct CellStyle {
    pub(crate) foreground: Hsla,
    pub(crate) background: Hsla,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    bold: bool,
    italic: bool,
}

impl CellStyle {
    pub(crate) fn of(cell: &Cell, palette: &Palette, overrides: &Colors) -> Self {
        let flags = cell.flags;
        let mut foreground = palette.resolve(cell.fg, overrides);
        let mut background = palette.resolve(cell.bg, overrides);
        if flags.contains(Flags::DIM) {
            foreground = foreground.opacity(DIM_OPACITY);
        }
        if flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut foreground, &mut background);
        }
        if flags.contains(Flags::HIDDEN) {
            foreground = background;
        }
        Self {
            foreground,
            background,
            underline: flags.intersects(Flags::ALL_UNDERLINES),
            strikethrough: flags.contains(Flags::STRIKEOUT),
            bold: flags.contains(Flags::BOLD),
            italic: flags.contains(Flags::ITALIC),
        }
    }

    pub(crate) fn under_cursor(self, palette: &Palette) -> Self {
        Self {
            foreground: palette.background(),
            background: palette.cursor(),
            ..self
        }
    }

    pub(crate) fn selected(self, palette: &Palette) -> Self {
        Self {
            background: palette.selection(),
            ..self
        }
    }

    pub(crate) fn is_decorated(&self) -> bool {
        self.underline || self.strikethrough
    }

    pub(crate) fn font(&self, base: &Font) -> Font {
        let mut font = base.clone();
        if self.bold {
            font = font.bold();
        }
        if self.italic {
            font = font.italic();
        }
        font
    }
}
