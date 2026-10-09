use gpui_kit::App;
use gpui_kit::component::ThemeConfig;

pub(super) fn drop_missing_fonts(config: &mut ThemeConfig, cx: &App) {
    if config.font_family.is_none() && config.mono_font_family.is_none() {
        return;
    }
    let installed = cx.text_system().all_font_names();
    for family in [&mut config.font_family, &mut config.mono_font_family] {
        if let Some(name) = family
            .as_ref()
            .filter(|name| !installed.iter().any(|font| font == name.as_ref()))
        {
            log::warn!("font {name} is not installed, using the platform default");
            *family = None;
        }
    }
}
