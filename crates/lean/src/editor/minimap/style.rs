use gpui_kit::*;

use crate::editor::settings::{
    MINIMAP_LINE_HEIGHT, MINIMAP_SLIDER, MINIMAP_SYNTAX_COLORS, MINIMAP_TEXT_OPACITY,
    MINIMAP_WIDTH, MinimapSlider, tab_size,
};

pub(super) const MAX_WIDTH: f32 = 200.;

const MIN_WIDTH: f32 = 60.;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct MinimapStyle {
    pub(super) line_height: f32,
    pub(super) syntax_colors: bool,
    pub(super) text_opacity: f32,
    pub(super) slider: MinimapSlider,
    pub(super) tab_columns: usize,
}

impl MinimapStyle {
    pub(super) fn read(cx: &App) -> Self {
        Self {
            line_height: MINIMAP_LINE_HEIGHT.get(cx).clamp(1., 4.),
            syntax_colors: MINIMAP_SYNTAX_COLORS.get(cx),
            text_opacity: MINIMAP_TEXT_OPACITY.get(cx).clamp(0.1, 1.),
            slider: MINIMAP_SLIDER.get(cx),
            tab_columns: tab_size(cx),
        }
    }

    pub(super) fn changes_rows(&self, other: &Self) -> bool {
        self.syntax_colors != other.syntax_colors || self.tab_columns != other.tab_columns
    }
}

pub(super) fn minimap_width(cx: &App) -> Pixels {
    px(MINIMAP_WIDTH.get(cx).clamp(MIN_WIDTH, MAX_WIDTH))
}
