use gpui_kit::App;

use super::registry::Registry;

pub(crate) struct Plugin {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) activate: fn(&mut Registry, &mut App),
}
