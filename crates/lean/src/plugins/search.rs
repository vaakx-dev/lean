mod engine;
mod panel;
mod query;
mod results;

use gpui_kit::{App, AppContext as _, Window};

use super::{AnyPanelView, Panel, Plugin, PluginContext, WorkspaceContext};
use panel::SearchPanel;

pub(super) const PLUGIN: Plugin = Plugin {
    id: "search",
    name: "Search",
    description: "Find text across the files in your project.",
    activate,
};

const PANEL: Panel = Panel {
    id: "search",
    title: "Search",
    icon: "icons/search.svg",
    order: 10,
    build,
};

fn activate(context: &mut PluginContext, _: &mut App) {
    context.register_panel(PANEL);
}

fn build(workspace: &WorkspaceContext, window: &mut Window, cx: &mut App) -> AnyPanelView {
    let project = workspace.project.clone();
    cx.new(|cx| SearchPanel::new(project, window, cx)).into()
}
