use std::path::Path;

use gpui_kit::{App, Global};

use super::file_action::FileAction;
use super::file_viewer::FileViewer;
use super::panel::Panel;
use super::service::StartService;
use super::status_item::StatusItem;

#[derive(Default)]
pub(in crate::plugins) struct Registry {
    active_plugins: Vec<&'static str>,
    viewers: Vec<FileViewer>,
    panels: Vec<Panel>,
    file_actions: Vec<FileAction>,
    services: Vec<StartService>,
    status_items: Vec<StatusItem>,
}

impl Global for Registry {}

impl Registry {
    pub(in crate::plugins) fn mark_active(&mut self, id: &'static str) {
        self.active_plugins.push(id);
    }

    pub(super) fn add_viewer(&mut self, viewer: FileViewer) {
        self.viewers.push(viewer);
        self.viewers
            .sort_by_key(|viewer| std::cmp::Reverse(viewer.priority));
    }

    pub(super) fn add_panel(&mut self, panel: Panel) {
        self.panels.push(panel);
        self.panels.sort_by_key(|panel| panel.order);
    }

    pub(super) fn add_file_action(&mut self, action: FileAction) {
        self.file_actions.push(action);
    }

    pub(super) fn add_service(&mut self, start: StartService) {
        self.services.push(start);
    }

    pub(super) fn add_status_item(&mut self, item: StatusItem) {
        self.status_items.push(item);
        self.status_items.sort_by_key(|item| item.order);
    }
}

pub(crate) fn viewer_for(path: &Path, cx: &App) -> Option<FileViewer> {
    cx.global::<Registry>()
        .viewers
        .iter()
        .find(|viewer| (viewer.handles)(path))
        .copied()
}

pub(crate) fn panels(cx: &App) -> &[Panel] {
    &cx.global::<Registry>().panels
}

pub(crate) fn status_items(cx: &App) -> &[StatusItem] {
    &cx.global::<Registry>().status_items
}

pub(super) fn services(cx: &App) -> Vec<StartService> {
    cx.global::<Registry>().services.clone()
}

pub(crate) fn file_actions(path: &Path, cx: &App) -> Vec<FileAction> {
    cx.global::<Registry>()
        .file_actions
        .iter()
        .filter(|action| (action.applies)(path))
        .copied()
        .collect()
}

pub(crate) fn is_active(id: &str, cx: &App) -> bool {
    cx.global::<Registry>()
        .active_plugins
        .iter()
        .any(|active| *active == id)
}
