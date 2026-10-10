use gpui_kit::component::input::TabSize;
use gpui_kit::{App, Pixels, SharedString};

use crate::editor::settings::{INSERT_SPACES, font_family, font_size, line_height, tab_size};

pub(super) struct CellFont {
    pub(super) family: SharedString,
    pub(super) size: Pixels,
    pub(super) line_height: f32,
}

impl CellFont {
    pub(super) fn line_pixels(&self) -> Pixels {
        self.size * self.line_height
    }
}

pub(super) fn cell_font(cx: &mut App) -> CellFont {
    CellFont {
        family: font_family(cx),
        size: font_size(cx),
        line_height: line_height(cx),
    }
}

pub(super) fn cell_tab_size(cx: &App) -> TabSize {
    TabSize {
        tab_size: tab_size(cx),
        hard_tabs: !INSERT_SPACES.get(cx),
    }
}
