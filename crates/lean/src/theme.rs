mod fonts;

use std::rc::Rc;

use anyhow::{Result, anyhow};
use gpui_kit::App;
use gpui_kit::component::scroll::ScrollbarMode;
use gpui_kit::component::{Theme, ThemeConfig, ThemeSet};

use crate::assets;

const THEME_PATH: &str = "themes/lean-dark.json";

pub(crate) fn init(cx: &mut App) {
    match load_theme() {
        Ok(mut config) => {
            fonts::drop_missing_fonts(&mut config, cx);
            Theme::update(cx, |theme| theme.apply_config(&Rc::new(config)));
        }
        Err(error) => log::error!("failed to load the Lean theme: {error:#}"),
    }
    Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);
}

fn load_theme() -> Result<ThemeConfig> {
    let theme_set: ThemeSet = serde_json::from_str(&assets::load_string(THEME_PATH)?)?;
    theme_set
        .themes
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("{THEME_PATH} contains no themes"))
}
