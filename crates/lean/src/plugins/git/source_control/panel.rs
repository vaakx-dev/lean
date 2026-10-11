use gpui_kit::*;

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
    let git = GitProject::for_workspace(workspace);
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
        if self.shows_changes(cx) {
            self.commit_message
                .update(cx, |input, cx| input.focus(window, cx));
        } else {
            window.focus(&self.focus_handle, cx);
        }
    }
}

impl SourceControl {
    fn shows_changes(&self, cx: &App) -> bool {
        let git = self.git.read(cx);
        git.error().is_none() && git.state().is_some()
    }

    pub(super) fn keep_focus_rendered(&self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.shows_changes(cx)
            && self
                .commit_message
                .focus_handle(cx)
                .contains_focused(window, cx)
        {
            window.focus(&self.focus_handle, cx);
        }
    }
}
