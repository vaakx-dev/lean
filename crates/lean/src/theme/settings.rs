mod page;

use gpui_kit::component::scroll::ScrollbarMode;
use serde::{Deserialize, Serialize};

use crate::settings::Setting;

pub(crate) use page::settings_page;

pub(crate) const DEFAULT_THEME: &str = "Lean Dark";
pub(crate) const DEFAULT_UI_FONT: &str = "Adwaita Sans";

pub(crate) const THEME: Setting<String> = Setting::new_with("appearance.theme", default_theme);
pub(crate) const ACCENT: Setting<String> = Setting::new_with("appearance.accent", default_accent);
pub(crate) const UI_FONT_FAMILY: Setting<String> =
    Setting::new_with("appearance.ui_font_family", default_ui_font);
pub(crate) const UI_FONT_SIZE: Setting<f32> = Setting::new("appearance.ui_font_size", 14.);
pub(crate) const RADIUS: Setting<u32> = Setting::new("appearance.radius", 6);
pub(crate) const SCROLLBARS: Setting<Scrollbars> =
    Setting::new("appearance.scrollbar", Scrollbars::Always);

pub(crate) const UI_FONT_SIZE_RANGE: (f32, f32) = (10., 24.);
pub(super) const RADIUS_MAX: u32 = 12;
pub(super) const RADIUS_LG_EXTRA: u32 = 4;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Scrollbars {
    Scrolling,
    Hover,
    Always,
}

impl Scrollbars {
    pub(super) fn mode(self) -> ScrollbarMode {
        match self {
            Self::Scrolling => ScrollbarMode::Scrolling,
            Self::Hover => ScrollbarMode::Hover,
            Self::Always => ScrollbarMode::Always,
        }
    }
}

fn default_theme() -> String {
    DEFAULT_THEME.into()
}

fn default_accent() -> String {
    "#d6aa72".into()
}

fn default_ui_font() -> String {
    DEFAULT_UI_FONT.into()
}
