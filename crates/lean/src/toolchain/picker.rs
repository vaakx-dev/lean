use std::path::PathBuf;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::*;

use super::{NO_TOOLCHAIN_LABEL, Toolchains};

pub(crate) fn toolchain_picker(
    toolchains: &Entity<Toolchains>,
    language: &'static str,
    cx: &App,
) -> Option<impl IntoElement + use<>> {
    let state = toolchains.read(cx);
    if !state.has_language(language) {
        return None;
    }
    let label = state.active_label(language);
    let toolchains = toolchains.clone();
    Some(
        Button::new("toolchain-picker")
            .ghost()
            .xsmall()
            .label(label)
            .dropdown_menu_with_anchor(Anchor::BottomRight, move |menu, _, cx| {
                build_menu(menu, &toolchains, language, cx)
            }),
    )
}

fn build_menu(
    menu: PopupMenu,
    toolchains: &Entity<Toolchains>,
    language: &'static str,
    cx: &App,
) -> PopupMenu {
    let state = toolchains.read(cx);
    let active = state.active(language).map(|toolchain| &toolchain.prefix);
    let none = choice(
        toolchains,
        language,
        None,
        NO_TOOLCHAIN_LABEL.into(),
        active.is_none(),
    );
    let choices = state.for_language(language).map(|toolchain| {
        choice(
            toolchains,
            language,
            Some(toolchain.prefix.clone()),
            toolchain.label(),
            active == Some(&toolchain.prefix),
        )
    });
    std::iter::once(none)
        .chain(choices)
        .fold(menu, |menu, item| menu.item(item))
        .separator()
        .item(refresh(toolchains))
}

fn choice(
    toolchains: &Entity<Toolchains>,
    language: &'static str,
    prefix: Option<PathBuf>,
    label: SharedString,
    checked: bool,
) -> PopupMenuItem {
    let toolchains = toolchains.clone();
    PopupMenuItem::new(label)
        .checked(checked)
        .on_click(move |_, _, cx| {
            let prefix = prefix.clone();
            toolchains.update(cx, |toolchains, cx| toolchains.select(language, prefix, cx))
        })
}

fn refresh(toolchains: &Entity<Toolchains>) -> PopupMenuItem {
    let toolchains = toolchains.clone();
    PopupMenuItem::new("Refresh")
        .on_click(move |_, _, cx| toolchains.update(cx, |toolchains, cx| toolchains.refresh(cx)))
}
