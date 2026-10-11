mod contributions;
mod file_decorations;
mod file_view;
mod panel;
mod plugin;
mod registry;
mod workspace_context;

pub(crate) use contributions::{FileAction, FileViewer, OpenRequest, StatusItem};
pub(crate) use file_decorations::{FileDecoration, FileDecorations};
pub(crate) use file_view::{AnyFileView, FileView, FileViewEvent, Snapshot};
pub(crate) use panel::{AnyPanelView, Panel, PanelEvent, PanelView};
pub(crate) use plugin::{Plugin, PluginContext};
pub(in crate::plugins) use registry::Registry;
pub(crate) use registry::{file_actions, is_active, panels, status_items, viewer_for};
pub(crate) use workspace_context::WorkspaceContext;
