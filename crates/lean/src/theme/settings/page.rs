use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};
use gpui_kit::{App, SharedString};

use super::{
    ACCENT, RADIUS, RADIUS_MAX, SCROLLBARS, Scrollbars, THEME, UI_FONT_FAMILY, UI_FONT_SIZE,
    UI_FONT_SIZE_RANGE,
};
use crate::plugins;
use crate::settings::ui::fields::{Number, color, font, group, number, select, switch};
use crate::theme::theme_names;
use crate::workspace::settings::{
    DEFAULT_PANEL, RESTORE_LAYOUT, SIDEBAR_WIDTH, SIDEBAR_WIDTH_RANGE, STATUS_BAR_VISIBLE,
    TITLE_BAR_SEARCH_BOX,
};

pub(crate) fn settings_page(cx: &App) -> SettingPage {
    SettingPage::new("Appearance")
        .icon(Icon::empty().path("icons/palette.svg"))
        .description("Theme, fonts and layout of the window.")
        .group(theme_group(cx))
        .group(interface_group())
        .group(layout_group(cx))
}

fn theme_group(cx: &App) -> SettingGroup {
    let themes: Vec<(String, SharedString)> = theme_names(cx)
        .into_iter()
        .map(|name| (name.to_string(), name))
        .collect();
    group(
        "Theme",
        [
            select(
                &THEME,
                "Theme",
                "Bundled themes plus JSON theme files in ~/.config/lean/themes.",
                themes,
            )
            .keywords(["color scheme", "colour scheme", "dark", "light"]),
            color(
                &ACCENT,
                "Accent colour",
                "Used for highlights, selections, focus rings and the active tab.",
            )
            .keywords(["accent color", "primary", "highlight"]),
        ],
    )
}

fn interface_group() -> SettingGroup {
    let (min_font_size, max_font_size) = UI_FONT_SIZE_RANGE;
    group(
        "Interface",
        [
            font(
                &UI_FONT_FAMILY,
                "UI font",
                "Font of menus, panels and tabs.",
                "System default",
            )
            .keywords(["typeface", "interface font"]),
            number(
                &UI_FONT_SIZE,
                "UI font size",
                "Text size of menus, panels and tabs.",
                Number::range(min_font_size.into(), max_font_size.into()).unit("px"),
            )
            .keywords(["text size", "zoom"]),
            number(
                &RADIUS,
                "Corner radius",
                "Roundness of buttons, inputs, tabs and panels.",
                Number::range(0., RADIUS_MAX.into()).unit("px"),
            )
            .keywords(["rounded corners", "border radius"]),
            select(
                &SCROLLBARS,
                "Scrollbars",
                "When scrollbars are visible.",
                [
                    (Scrollbars::Scrolling, "While scrolling"),
                    (Scrollbars::Hover, "On hover"),
                    (Scrollbars::Always, "Always"),
                ],
            )
            .keywords(["scroll bar", "overlay"]),
        ],
    )
}

fn layout_group(cx: &App) -> SettingGroup {
    let panels: Vec<(String, SharedString)> = plugins::panels(cx)
        .iter()
        .map(|panel| (panel.id.to_string(), panel.title.into()))
        .collect();
    let (min_width, max_width) = SIDEBAR_WIDTH_RANGE;
    group(
        "Layout",
        [
            switch(
                &RESTORE_LAYOUT,
                "Remember layout",
                "Reopen with the sidebar, its panel and width, and the terminal as they were.",
            )
            .keywords(["restore", "persist", "panels"]),
            select(
                &DEFAULT_PANEL,
                "Default sidebar panel",
                "Panel shown when a project opens.",
                panels,
            )
            .keywords(["explorer", "search", "source control", "activity bar"]),
            number(
                &SIDEBAR_WIDTH,
                "Sidebar width",
                "Starting width of the sidebar. Drag its edge to resize it.",
                Number::range(min_width.into(), max_width.into()).unit("px"),
            ),
            switch(
                &STATUS_BAR_VISIBLE,
                "Show status bar",
                "The bar along the bottom of the window.",
            )
            .keywords(["statusbar", "footer"]),
            switch(
                &TITLE_BAR_SEARCH_BOX,
                "Title bar search box",
                "Show the search box in the middle of the title bar.",
            )
            .keywords(["command center", "titlebar"]),
        ],
    )
}
