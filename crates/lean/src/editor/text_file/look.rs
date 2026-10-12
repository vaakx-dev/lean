use gpui_kit::*;

use crate::editor::minimap::minimap_width;
use crate::editor::settings::{font_family, font_size, line_height};

#[derive(Clone, PartialEq)]
pub(super) struct EditorLook {
    pub(super) font_family: SharedString,
    pub(super) font_size: Pixels,
    pub(super) line_height: f32,
    pub(super) minimap_width: Pixels,
}

impl EditorLook {
    pub(super) fn read(cx: &mut App) -> Self {
        Self {
            font_family: font_family(cx),
            font_size: font_size(cx),
            line_height: line_height(cx),
            minimap_width: minimap_width(cx),
        }
    }
}
