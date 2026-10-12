use gpui_kit::App;
#[cfg(any(feature = "media", feature = "browser"))]
use gpui_kit::component::Icon;
use gpui_kit::component::setting::SettingPage;

use crate::{app, editor, explorer, files, plugins, theme, toolchain};

pub(super) struct Pages {
    pub(super) all: Vec<SettingPage>,
    pub(super) keybindings: usize,
}

pub(super) fn settings_pages(cx: &App) -> Pages {
    let mut all = feature_pages(cx);
    let keybindings = all.len();
    all.push(app::keybindings::settings_page(cx));
    all.push(plugins::settings::settings_page(cx));
    Pages { all, keybindings }
}

fn feature_pages(cx: &App) -> Vec<SettingPage> {
    let mut pages = vec![
        app::settings::settings_page(cx),
        theme::settings::settings_page(cx),
        editor::settings::settings_page(cx),
        files::settings::settings_page(cx),
        explorer::settings::settings_page(cx),
    ];
    #[cfg(feature = "terminal")]
    pages.push(crate::terminal::settings::settings_page(cx));
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
