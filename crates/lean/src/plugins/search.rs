mod config;
mod engine;
mod panel;
mod query;
mod results;
pub(crate) mod settings;

use gpui_kit::{App, AppContext as _, KeyBinding, Window};

use super::{AnyPanelView, Panel, Plugin, Registry, WorkspaceContext};
use crate::app::actions::ShowPanel;
use crate::app::keymap;
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

fn activate(registry: &mut Registry, cx: &mut App) {
    registry.register_panel(PANEL);
    keymap::bind_command(
        cx,
        "lean::FindInFiles",
        "Find in Files",
        [KeyBinding::new(
            "ctrl-shift-f",
            ShowPanel { id: PANEL.id },
            None,
        )],
    );
}

fn build(workspace: &WorkspaceContext, window: &mut Window, cx: &mut App) -> AnyPanelView {
    let project = workspace.project.clone();
    cx.new(|cx| SearchPanel::new(project, window, cx)).into()
}
