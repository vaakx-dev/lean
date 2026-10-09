use gpui_kit::{App, AppContext as _, Window};

use super::Explorer;
use crate::plugins::{AnyPanelView, Panel, PanelView, WorkspaceContext};

pub(crate) const PANEL: Panel = Panel {
    id: "explorer",
    title: "Explorer",
    icon: "icons/files.svg",
    order: 0,
    build,
};

fn build(workspace: &WorkspaceContext, window: &mut Window, cx: &mut App) -> AnyPanelView {
    cx.new(|cx| Explorer::new(workspace, window, cx)).into()
}

impl PanelView for Explorer {}
