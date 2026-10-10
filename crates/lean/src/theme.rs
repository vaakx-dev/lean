mod accent;
mod appearance;
mod catalog;
mod fonts;
pub(crate) mod settings;
mod user_themes;

use gpui_kit::App;

pub(crate) use catalog::theme_names;
pub(crate) use fonts::installed_font_names;

pub(crate) fn init(cx: &mut App) {
    catalog::load(cx);
    appearance::apply_current(cx);
    appearance::follow_settings(cx);
    user_themes::watch(cx);
}
