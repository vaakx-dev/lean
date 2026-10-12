use std::collections::HashSet;
use std::rc::Rc;

use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, Global, SharedString};

struct InstalledFonts {
    sorted: Rc<[SharedString]>,
    lookup: HashSet<SharedString>,
}

impl Global for InstalledFonts {}

fn installed_fonts(cx: &mut App) -> &InstalledFonts {
    if !cx.has_global::<InstalledFonts>() {
        let mut names = cx.text_system().all_font_names();
        names.sort_by_cached_key(|name| name.to_lowercase());
        names.dedup();
        let sorted: Rc<[SharedString]> = names.into_iter().map(SharedString::from).collect();
        let lookup = sorted.iter().cloned().collect();
        cx.set_global(InstalledFonts { sorted, lookup });
    }
    cx.global::<InstalledFonts>()
}

pub(crate) fn installed_font_names(cx: &mut App) -> Rc<[SharedString]> {
    installed_fonts(cx).sorted.clone()
}

pub(crate) fn installed_font(name: &str, cx: &mut App) -> Option<SharedString> {
    installed_fonts(cx).lookup.get(name).cloned()
}

pub(super) fn drop_missing_fonts(config: &mut ThemeConfig, cx: &mut App) {
    if config.font_family.is_none() && config.mono_font_family.is_none() {
        return;
    }
    for family in [&mut config.font_family, &mut config.mono_font_family] {
        if let Some(name) = family.clone() {
            if installed_font(&name, cx).is_none() {
                log::warn!("font {name} is not installed, using the platform default");
                *family = None;
            }
        }
    }
}
