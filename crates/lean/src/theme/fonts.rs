use std::rc::Rc;

use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, Global, SharedString};

struct InstalledFonts(Rc<[SharedString]>);

impl Global for InstalledFonts {}

pub(crate) fn installed_font_names(cx: &mut App) -> Rc<[SharedString]> {
    if let Some(fonts) = cx.try_global::<InstalledFonts>() {
        return fonts.0.clone();
    }
    let mut names = cx.text_system().all_font_names();
    names.sort_by_key(|name| name.to_lowercase());
    names.dedup();
    let fonts: Rc<[SharedString]> = names.into_iter().map(SharedString::from).collect();
    cx.set_global(InstalledFonts(fonts.clone()));
    fonts
}

pub(super) fn drop_missing_fonts(config: &mut ThemeConfig, cx: &mut App) {
    if config.font_family.is_none() && config.mono_font_family.is_none() {
        return;
    }
    let installed = installed_font_names(cx);
    for family in [&mut config.font_family, &mut config.mono_font_family] {
        if let Some(name) = family.as_ref().filter(|name| !installed.contains(*name)) {
            log::warn!("font {name} is not installed, using the platform default");
            *family = None;
        }
    }
}
