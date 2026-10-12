#[cfg(feature = "browser")]
pub(crate) mod browser;
#[cfg(feature = "git")]
pub(crate) mod git;
mod host;
#[cfg(feature = "media")]
pub(crate) mod media;
#[cfg(feature = "notebook")]
pub(crate) mod notebook;
#[cfg(feature = "search")]
pub(crate) mod search;
pub(crate) mod settings;

use gpui_kit::App;

use crate::settings::plugin_enabled;
use crate::{editor, explorer};

pub(crate) use gpui_kit::component::input::GutterMarker;
pub(crate) use host::{
    AnyFileView, AnyPanelView, FileAction, FileDecoration, FileDecorations, FileView,
    FileViewEvent, FileViewer, OpenRequest, Panel, PanelEvent, PanelView, Plugin, Registry,
    Snapshot, StatusItem, WorkspaceContext, file_actions, panels, status_items, viewer_for,
};

const PLUGINS: &[Plugin] = &[
    #[cfg(feature = "git")]
    git::PLUGIN,
    #[cfg(feature = "search")]
    search::PLUGIN,
    #[cfg(feature = "media")]
    media::PLUGIN,
    #[cfg(feature = "browser")]
    browser::PLUGIN,
    #[cfg(feature = "notebook")]
    notebook::PLUGIN,
];

pub(crate) fn init(cx: &mut App) {
    let mut registry = Registry::default();
    registry.register_viewer(editor::TEXT_VIEWER);
    registry.register_panel(explorer::PANEL);
    for plugin in PLUGINS {
        if plugin_enabled(plugin.id, cx) {
            (plugin.activate)(&mut registry, cx);
            registry.mark_active(plugin.id);
        }
    }
    cx.set_global(registry);
}
