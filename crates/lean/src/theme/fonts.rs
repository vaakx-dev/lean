use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, SharedString};

pub(super) fn drop_missing_fonts(config: &mut ThemeConfig, cx: &App) {
    let installed = cx.text_system().all_font_names();
    for family in [&mut config.font_family, &mut config.mono_font_family] {
        if let Some(name) = family
            .as_ref()
            .filter(|name| !is_installed(name, &installed))
        {
            log::warn!("font {name} is not installed, using the platform default");
            *family = None;
        }
    }
}

fn is_installed(name: &SharedString, installed: &[String]) -> bool {
    installed.iter().any(|font| font == name.as_ref())
}
