use std::rc::Rc;

use gpui_kit::component::{ThemeConfig, ThemeRegistry, ThemeSet};
use gpui_kit::{App, AssetSource as _, Global, SharedString};

use super::user_themes;
use crate::assets::{self, Assets};

const BUNDLED_FOLDER: &str = "themes/";

struct ThemeCatalog {
    themes: Vec<Rc<ThemeConfig>>,
}

impl Global for ThemeCatalog {}

pub(super) fn load(cx: &mut App) {
    let registry = ThemeRegistry::global(cx);
    let kit_themes = [
        registry.default_dark_theme().clone(),
        registry.default_light_theme().clone(),
    ];
    let mut themes = Vec::new();
    for theme in bundled_themes() {
        add(&mut themes, Rc::new(theme));
    }
    for theme in kit_themes {
        add(&mut themes, theme);
    }
    for theme in user_themes::load() {
        add(&mut themes, Rc::new(theme));
    }
    cx.set_global(ThemeCatalog { themes });
}

pub(crate) fn theme_names(cx: &App) -> Vec<SharedString> {
    cx.try_global::<ThemeCatalog>()
        .map(|catalog| {
            catalog
                .themes
                .iter()
                .map(|theme| theme.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn find(name: &str, cx: &App) -> Option<Rc<ThemeConfig>> {
    cx.try_global::<ThemeCatalog>()?
        .themes
        .iter()
        .find(|theme| theme.name.as_ref() == name)
        .cloned()
}

pub(super) fn parse_theme_set(contents: &str) -> serde_json::Result<Vec<ThemeConfig>> {
    serde_json::from_str::<ThemeSet>(contents).map(|set| set.themes)
}

fn add(themes: &mut Vec<Rc<ThemeConfig>>, theme: Rc<ThemeConfig>) {
    match themes.iter_mut().find(|known| known.name == theme.name) {
        Some(known) => *known = theme,
        None => themes.push(theme),
    }
}

fn bundled_themes() -> Vec<ThemeConfig> {
    let paths = Assets.list(BUNDLED_FOLDER).unwrap_or_else(|error| {
        log::error!("failed to list the bundled themes: {error:#}");
        Vec::new()
    });
    paths
        .iter()
        .filter(|path| path.starts_with(BUNDLED_FOLDER) && path.ends_with(".json"))
        .flat_map(|path| match bundled_theme(path) {
            Ok(themes) => themes,
            Err(error) => {
                log::error!("failed to load the bundled theme {path}: {error:#}");
                Vec::new()
            }
        })
        .collect()
}

fn bundled_theme(path: &str) -> anyhow::Result<Vec<ThemeConfig>> {
    Ok(parse_theme_set(&assets::load_string(path)?)?)
}
