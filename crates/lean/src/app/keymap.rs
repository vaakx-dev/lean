mod apply;
mod defaults;
mod names;
mod overrides;
mod registry;
mod rows;

use gpui_kit::App;

use crate::editor::settings::font;
#[cfg(feature = "terminal")]
use crate::terminal::settings::PASSTHROUGH_KEYS;

pub(crate) use defaults::{GoToLine, RevealActiveFile};
pub(crate) use overrides::{KEYMAP, remove_keys, reset_keys, set_keys};
#[cfg(any(feature = "git", feature = "search"))]
pub(crate) use registry::bind_command;
pub(crate) use registry::{apply_bindings, bind_defaults};
pub(crate) use rows::{Row, Table, rows};

pub(super) fn bind_keys(cx: &mut App) {
    font::init(cx);
    defaults::bind(cx);
    KEYMAP
        .observe_app(cx, |_, cx| {
            apply::apply(cx);
            super::set_menus(cx);
        })
        .detach();
    #[cfg(feature = "terminal")]
    PASSTHROUGH_KEYS
        .observe_app(cx, |_, cx| apply::apply(cx))
        .detach();
}
