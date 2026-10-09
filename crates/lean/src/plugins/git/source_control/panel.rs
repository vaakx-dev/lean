use gpui_kit::{App, AppContext as _, Context, Window};

use super::SourceControl;
use crate::plugins::git::git_project::GitProject;
use crate::plugins::{AnyPanelView, Panel, PanelView, WorkspaceContext};

pub(crate) const PANEL: Panel = Panel {
    id: "source_control",
    title: "Source Control",
    icon: "icons/git-branch.svg",
    order: 20,
    build,
};

fn build(workspace: &WorkspaceContext, window: &mut Window, cx: &mut App) -> AnyPanelView {
    let git = GitProject::for_workspace(workspace, cx);
    cx.new(|cx| SourceControl::new(git, window, cx)).into()
}

impl PanelView for SourceControl {
    fn badge(&self, cx: &App) -> usize {
        self.git
            .read(cx)
            .state()
            .map_or(0, |state| state.statuses.len())
    }

    fn focus_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
    }
}
