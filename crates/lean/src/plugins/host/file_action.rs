use std::path::Path;

use gpui_kit::{App, Window};

#[derive(Clone, Copy)]
pub(crate) struct FileAction {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) applies: fn(&Path, &App) -> bool,
    pub(crate) run: fn(&Path, &mut Window, &mut App),
}
