mod apply;
mod defaults;
mod font_size;
mod names;
mod overrides;
mod registry;
mod rows;

use gpui_kit::App;

pub(crate) use defaults::{GoToLine, RevealActiveFile};
pub(crate) use overrides::{KEYMAP, remove_keys, reset_keys, set_keys};
#[cfg(any(feature = "git", feature = "search"))]
pub(crate) use registry::bind_command;
pub(crate) use registry::bind_defaults;
pub(crate) use rows::{Row, conflicts, rows};

pub(super) fn bind_keys(cx: &mut App) {
    font_size::init(cx);
    defaults::bind(cx);
    KEYMAP
        .observe_app(cx, |_, cx| {
            apply::apply(cx);
            super::set_menus(cx);
        })
        .detach();
}
