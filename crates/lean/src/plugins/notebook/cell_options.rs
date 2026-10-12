use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::TabSize;
use gpui_kit::{App, Pixels, SharedString};

use crate::editor::settings::{FONT_FAMILY, INSERT_SPACES, font_size, line_height, tab_size};
use crate::theme::installed_font;

pub(super) struct CellFont {
    family: Option<SharedString>,
    pub(super) size: Pixels,
    pub(super) line_height: f32,
}

impl CellFont {
    pub(super) fn family(&self, cx: &App) -> SharedString {
        self.family
            .clone()
            .unwrap_or_else(|| cx.theme().mono_font_family.clone())
    }

    pub(super) fn line_pixels(&self) -> Pixels {
        self.size * self.line_height
    }
}

pub(super) fn cell_font(cx: &mut App) -> CellFont {
    CellFont {
        family: FONT_FAMILY
            .get(cx)
            .and_then(|name| installed_font(&name, cx)),
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
