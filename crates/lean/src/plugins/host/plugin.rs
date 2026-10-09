use gpui_kit::App;

use super::file_action::FileAction;
use super::file_viewer::FileViewer;
use super::panel::Panel;
use super::registry::Registry;
use super::service::StartService;
use super::status_item::StatusItem;

pub(crate) struct Plugin {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) activate: fn(&mut PluginContext, &mut App),
}

pub(crate) struct PluginContext<'a> {
    registry: &'a mut Registry,
}

impl<'a> PluginContext<'a> {
    pub(in crate::plugins) fn new(registry: &'a mut Registry) -> Self {
        Self { registry }
    }

    pub(crate) fn register_viewer(&mut self, viewer: FileViewer) {
        self.registry.add_viewer(viewer);
    }

    pub(crate) fn register_panel(&mut self, panel: Panel) {
        self.registry.add_panel(panel);
    }

    pub(crate) fn register_file_action(&mut self, action: FileAction) {
        self.registry.add_file_action(action);
    }

    pub(crate) fn register_service(&mut self, start: StartService) {
        self.registry.add_service(start);
    }

    pub(crate) fn register_status_item(&mut self, item: StatusItem) {
        self.registry.add_status_item(item);
    }
}
