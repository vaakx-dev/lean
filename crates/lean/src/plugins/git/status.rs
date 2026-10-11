use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, Theme, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::git_project::GitProject;
use super::repo::GitState;
use super::settings::STATUS_BAR_BRANCH;
use super::source_control::PANEL;
use crate::app::actions::ShowPanel;
use crate::plugins::{StatusItem, WorkspaceContext};
use crate::workspace::status_item;

pub(super) const STATUS_ITEM: StatusItem = StatusItem { order: 0, build };

fn build(workspace: &WorkspaceContext, _: &mut Window, cx: &mut App) -> AnyView {
    let git = GitProject::for_workspace(workspace);
    cx.new(|cx| GitStatus::new(git, cx)).into()
}

struct GitStatus {
    git: Entity<GitProject>,
    show_branch: bool,
    _subscriptions: [Subscription; 2],
}

impl GitStatus {
    fn new(git: Entity<GitProject>, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.observe(&git, |_, _, cx| cx.notify()),
            STATUS_BAR_BRANCH.observe(cx, |status, show, cx| {
                status.show_branch = show;
                cx.notify();
            }),
        ];
        Self {
            git,
            show_branch: STATUS_BAR_BRANCH.get(cx),
            _subscriptions: subscriptions,
        }
    }
}

impl Render for GitStatus {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let git = self.git.read(cx);
        h_flex()
            .h_full()
            .gap_1()
            .when_some(git.error(), |bar, error| {
                bar.child(error_item(error.message.clone(), theme))
            })
            .when_some(git.state().filter(|_| self.show_branch), |bar, state| {
                bar.child(branch_item(state, theme))
            })
    }
}

fn branch_item(state: &GitState, theme: &Theme) -> Stateful<Div> {
    let dirty_marker = if state.is_dirty() { "*" } else { "" };
    status_item("branch", theme)
        .child(Icon::empty().path("icons/git-branch.svg").size_4())
        .child(format!("{}{dirty_marker}", state.branch))
}

fn error_item(message: SharedString, theme: &Theme) -> Stateful<Div> {
    status_item("git-error", theme)
        .cursor_pointer()
        .text_color(theme.warning)
        .child(Icon::empty().path("icons/triangle-alert.svg").size_4())
        .child("Git error")
        .tooltip(move |window, cx| Tooltip::new(message.clone()).build(window, cx))
        .on_click(|_, window, cx| window.dispatch_action(Box::new(ShowPanel { id: PANEL.id }), cx))
}
