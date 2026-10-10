use std::rc::Rc;

use gpui_kit::App;
use gpui_kit::component::Theme;

use super::accent::apply_accent;
use super::catalog;
use super::fonts::drop_missing_fonts;
use super::settings::{
    ACCENT, DEFAULT_THEME, RADIUS, RADIUS_LG_EXTRA, RADIUS_MAX, SCROLLBARS, Scrollbars, THEME,
    UI_FONT_FAMILY, UI_FONT_SIZE, UI_FONT_SIZE_RANGE,
};
use crate::settings::Settings;

#[derive(Clone, PartialEq)]
struct Appearance {
    theme: String,
    accent: Option<String>,
    font_family: String,
    font_size: f32,
    radius: u32,
    scrollbars: Scrollbars,
}

impl Appearance {
    fn current(cx: &App) -> Self {
        Self {
            theme: THEME.get(cx),
            accent: ACCENT.is_modified(cx).then(|| ACCENT.get(cx)),
            font_family: UI_FONT_FAMILY.get(cx),
            font_size: UI_FONT_SIZE.get(cx),
            radius: RADIUS.get(cx),
            scrollbars: SCROLLBARS.get(cx),
        }
    }
}

pub(super) fn apply_current(cx: &mut App) {
    apply(&Appearance::current(cx), cx);
}

pub(super) fn follow_settings(cx: &mut App) {
    let mut applied = Appearance::current(cx);
    cx.observe_global::<Settings>(move |cx| {
        let next = Appearance::current(cx);
        if next != applied {
            apply(&next, cx);
            applied = next;
        }
    })
    .detach();
}

fn apply(appearance: &Appearance, cx: &mut App) {
    let base = catalog::find(&appearance.theme, cx).or_else(|| {
        log::warn!(
            "theme {} was not found, using {DEFAULT_THEME}",
            appearance.theme
        );
        catalog::find(DEFAULT_THEME, cx)
    });
    let Some(base) = base else {
        log::error!("the {DEFAULT_THEME} theme is missing");
        return;
    };
    let mut config = (*base).clone();
    if let Some(accent) = &appearance.accent {
        apply_accent(&mut config.colors, accent);
    }
    let (min_font_size, max_font_size) = UI_FONT_SIZE_RANGE;
    let radius = appearance.radius.min(RADIUS_MAX);
    config.font_family = Some(appearance.font_family.clone().into());
    config.font_size = Some(appearance.font_size.clamp(min_font_size, max_font_size));
    config.radius = Some(radius as usize);
    config.radius_lg = Some((radius + RADIUS_LG_EXTRA) as usize);
    drop_missing_fonts(&mut config, cx);
    if config.font_family.is_none() {
        config.font_family = base.font_family.clone();
        drop_missing_fonts(&mut config, cx);
    }
    let config = Rc::new(config);
    let scrollbar_mode = appearance.scrollbars.mode();
    Theme::update(cx, |theme| {
        theme.apply_config(&config);
        theme.scrollbar_mode = scrollbar_mode;
    });
}
