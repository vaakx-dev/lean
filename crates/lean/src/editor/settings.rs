pub(crate) mod font;
mod page;

use gpui_kit::App;
use serde::{Deserialize, Serialize};

use crate::settings::Setting;

pub(crate) use font::{font_family, font_size, line_height};
pub(crate) use page::settings_page;

pub(crate) const FONT_FAMILY: Setting<Option<String>> = Setting::new("editor.font_family", None);
pub(crate) const FONT_SIZE: Setting<f32> = Setting::new("editor.font_size", 14.);
pub(crate) const LINE_HEIGHT: Setting<f32> = Setting::new("editor.line_height", 1.5);

pub(crate) const TAB_SIZE: Setting<u32> = Setting::new("editor.tab_size", 4);
pub(crate) const INSERT_SPACES: Setting<bool> = Setting::new("editor.insert_spaces", true);
pub(crate) const SMART_INDENT: Setting<bool> = Setting::new("editor.smart_indent", true);
pub(crate) const AUTO_CLOSE_BRACKETS: Setting<bool> =
    Setting::new("editor.auto_close_brackets", true);

pub(crate) const LINE_NUMBERS: Setting<bool> = Setting::new("editor.line_numbers", true);
pub(crate) const INDENT_GUIDES: Setting<bool> = Setting::new("editor.indent_guides", true);
pub(crate) const CODE_FOLDING: Setting<bool> = Setting::new("editor.code_folding", true);
pub(crate) const SOFT_WRAP: Setting<bool> = Setting::new("editor.soft_wrap", false);
pub(crate) const SHOW_WHITESPACE: Setting<bool> = Setting::new("editor.show_whitespace", false);
pub(crate) const PREVIEW_TABS: Setting<bool> = Setting::new("editor.preview_tabs", false);

pub(crate) const MINIMAP_ENABLED: Setting<bool> = Setting::new("editor.minimap.enabled", true);
pub(crate) const MINIMAP_WIDTH: Setting<f32> = Setting::new("editor.minimap.width", 100.);
pub(crate) const MINIMAP_LINE_HEIGHT: Setting<f32> = Setting::new("editor.minimap.line_height", 2.);
pub(crate) const MINIMAP_SYNTAX_COLORS: Setting<bool> =
    Setting::new("editor.minimap.syntax_colors", true);
pub(crate) const MINIMAP_TEXT_OPACITY: Setting<f32> =
    Setting::new("editor.minimap.text_opacity", 0.6);
pub(crate) const MINIMAP_SLIDER: Setting<MinimapSlider> =
    Setting::new("editor.minimap.slider", MinimapSlider::Always);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MinimapSlider {
    Always,
    Hover,
    Never,
}

pub(crate) fn tab_size(cx: &App) -> usize {
    TAB_SIZE.get(cx).clamp(1, 8) as usize
}
