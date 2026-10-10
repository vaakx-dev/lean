use gpui_kit::App;
#[cfg(any(feature = "media", feature = "browser"))]
use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;

use crate::{app, editor, explorer, files, plugins, terminal, theme, toolchain};

pub(super) fn settings_pages(cx: &App) -> Vec<SettingPage> {
    let mut pages = feature_pages(cx);
    pages.push(app::keybindings::settings_page(cx));
    pages.push(plugins::settings::settings_page(cx));
    pages
}

pub(super) fn keybindings_page_index(cx: &App) -> usize {
    feature_pages(cx).len()
}

fn feature_pages(cx: &App) -> Vec<SettingPage> {
    let mut pages = vec![
        app::settings::settings_page(cx),
        theme::settings::settings_page(cx),
        editor::settings::settings_page(cx),
        files::settings::settings_page(cx),
        explorer::settings::settings_page(cx),
        terminal::settings::settings_page(cx),
    ];
    #[cfg(feature = "git")]
    pages.push(plugins::git::settings::settings_page(cx));
    #[cfg(feature = "search")]
    pages.push(plugins::search::settings::settings_page(cx));
    #[cfg(feature = "notebook")]
    pages.push(plugins::notebook::settings::settings_page(cx));
    #[cfg(any(feature = "media", feature = "browser"))]
    pages.push(media_page(cx));
    pages.push(toolchain::settings::settings_page(cx));
    pages
}

#[cfg(any(feature = "media", feature = "browser"))]
fn media_page(cx: &App) -> SettingPage {
    let page = SettingPage::new("Media")
        .icon(Icon::empty().path("icons/image.svg"))
        .description("Images, SVG previews and opening files in a browser.");
    #[cfg(feature = "media")]
    let page = page.groups(plugins::media::settings::settings_groups(cx));
    #[cfg(feature = "browser")]
    let page = page.groups(plugins::browser::settings::settings_groups(cx));
    page
}
