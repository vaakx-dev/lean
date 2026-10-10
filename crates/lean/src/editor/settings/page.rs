use gpui_kit::App;
use gpui_kit::component::Icon;
use gpui_kit::component::setting::{SettingGroup, SettingPage};

use super::{
    AUTO_CLOSE_BRACKETS, CODE_FOLDING, FONT_FAMILY, FONT_SIZE, INDENT_GUIDES, INSERT_SPACES,
    LINE_HEIGHT, LINE_NUMBERS, MINIMAP_ENABLED, MINIMAP_LINE_HEIGHT, MINIMAP_SLIDER,
    MINIMAP_SYNTAX_COLORS, MINIMAP_TEXT_OPACITY, MINIMAP_WIDTH, MinimapSlider, PREVIEW_TABS,
    SHOW_WHITESPACE, SMART_INDENT, SOFT_WRAP, TAB_SIZE,
};
use crate::settings::ui::fields::{Number, font, group, number, select, switch};

pub(crate) fn settings_page(_: &App) -> SettingPage {
    SettingPage::new("Editor")
        .icon(Icon::empty().path("icons/file-code.svg"))
        .description("Font, indentation, display and minimap for text files.")
        .group(font_group())
        .group(typing_group())
        .group(display_group())
        .group(minimap_group())
}

fn font_group() -> SettingGroup {
    group(
        "Font",
        [
            font(
                &FONT_FAMILY,
                "Editor font",
                "Also used by the terminal, notebooks and diffs unless they set their own.",
                "JetBrainsMono Nerd Font",
            )
            .keywords(["font family", "monospace", "typeface"]),
            number(
                &FONT_SIZE,
                "Font size",
                "Text size in the editor. Also the default for the terminal.",
                Number::range(8., 32.).unit("px"),
            )
            .keywords(["zoom", "text size"]),
            number(
                &LINE_HEIGHT,
                "Line height",
                "Height of each line as a multiple of the font size.",
                Number::range(1., 2.5).step(0.1).unit("×"),
            )
            .keywords(["leading", "line spacing"]),
        ],
    )
}

fn typing_group() -> SettingGroup {
    group(
        "Indentation & typing",
        [
            number(
                &TAB_SIZE,
                "Tab size",
                "Columns per indent level, also used for space indents.",
                Number::range(1., 8.).unit("cols"),
            )
            .keywords(["indent width", "tab width"]),
            switch(
                &INSERT_SPACES,
                "Insert spaces",
                "Press Tab to type spaces instead of a tab character.",
            )
            .keywords(["soft tabs", "hard tabs", "indent with tabs"]),
            switch(
                &SMART_INDENT,
                "Keep indentation on Enter",
                "New lines start at the indentation of the line above.",
            )
            .keywords(["auto indent", "smart indent"]),
            switch(
                &AUTO_CLOSE_BRACKETS,
                "Auto-close brackets and quotes",
                "Insert the closing character as you type the opening one.",
            )
            .keywords(["auto pair", "parentheses", "brackets"]),
        ],
    )
}

fn display_group() -> SettingGroup {
    group(
        "Display",
        [
            switch(
                &LINE_NUMBERS,
                "Line numbers",
                "Show a number beside each line.",
            )
            .keywords(["gutter"]),
            switch(
                &INDENT_GUIDES,
                "Indent guides",
                "Draw a vertical line at each indent level.",
            ),
            switch(
                &CODE_FOLDING,
                "Code folding",
                "Collapse blocks from the gutter.",
            )
            .keywords(["fold", "collapse"]),
            switch(
                &SOFT_WRAP,
                "Wrap long lines",
                "Wrap at the editor edge instead of scrolling sideways.",
            )
            .keywords(["word wrap", "soft wrap"]),
            switch(
                &SHOW_WHITESPACE,
                "Show whitespace",
                "Draw spaces and tabs as faint markers.",
            )
            .keywords(["invisibles", "render whitespace"]),
            switch(
                &PREVIEW_TABS,
                "Preview tabs",
                "A single click opens a tab that the next click replaces. Editing or double-clicking keeps it.",
            )
            .keywords(["tabs", "single click", "pin"]),
        ],
    )
}

fn minimap_group() -> SettingGroup {
    let enabled = |cx: &App| MINIMAP_ENABLED.get(cx);
    group(
        "Minimap",
        [
            switch(
                &MINIMAP_ENABLED,
                "Show minimap",
                "Turning it off also stops background minimap work.",
            )
            .keywords(["overview", "code map"]),
            number(
                &MINIMAP_WIDTH,
                "Minimap width",
                "Width of the minimap beside the editor.",
                Number::range(60., 200.).step(10.).unit("px"),
            )
            .enabled_when(enabled),
            number(
                &MINIMAP_LINE_HEIGHT,
                "Minimap line height",
                "Height of each line in the minimap.",
                Number::range(1., 4.).step(0.5).unit("px"),
            )
            .enabled_when(enabled),
            switch(
                &MINIMAP_SYNTAX_COLORS,
                "Syntax colours",
                "Colour the minimap like the code. Off draws every line in one colour.",
            )
            .keywords(["highlighting", "colors"])
            .enabled_when(enabled),
            number(
                &MINIMAP_TEXT_OPACITY,
                "Text opacity",
                "How strongly the minimap draws the text.",
                Number::range(0.1, 1.).step(0.05),
            )
            .enabled_when(enabled),
            select(
                &MINIMAP_SLIDER,
                "Viewport slider",
                "When to show the box marking the visible part of the file.",
                [
                    (MinimapSlider::Always, "Always"),
                    (MinimapSlider::Hover, "On hover"),
                    (MinimapSlider::Never, "Never"),
                ],
            )
            .keywords(["scrollbar", "viewport"])
            .enabled_when(enabled),
        ],
    )
}
