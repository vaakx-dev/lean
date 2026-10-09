use super::PluginContext;
use crate::{editor, explorer};

pub(super) fn register(context: &mut PluginContext) {
    context.register_viewer(editor::TEXT_VIEWER);
    context.register_panel(explorer::PANEL);
}
