use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, Theme, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use super::sidebar::SidebarView;

const STATUS_BAR_HEIGHT: Pixels = px(28.);

impl Workspace {
    pub(super) fn render_status_bar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .h(STATUS_BAR_HEIGHT)
            .flex_shrink_0()
            .px_2()
            .py_0p5()
            .justify_between()
            .text_sm()
            .text_color(theme.muted_foreground)
            .child(self.render_project_status(cx))
            .child(self.render_editor_status(cx))
    }

    fn render_project_status(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let project = self.project.read(cx);
        h_flex()
            .h_full()
            .gap_1()
            .when_some(project.git_error(), |bar, error| {
                bar.child(self.render_git_error_status(error.message.clone(), cx))
            })
            .when_some(project.git(), |bar, git| {
                let dirty_marker = if git.is_dirty() { "*" } else { "" };
                bar.child(
                    status_item("branch", theme)
                        .child(Icon::empty().path("icons/git-branch.svg").size_4())
                        .child(format!("{}{dirty_marker}", git.branch)),
                )
            })
            .child(
                status_item("problems", theme)
                    .child(Icon::empty().path("icons/circle-x.svg").size_4())
                    .child("0")
                    .child(Icon::empty().path("icons/triangle-alert.svg").size_4())
                    .child("0"),
            )
    }

    fn render_git_error_status(&self, message: SharedString, cx: &Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        status_item("git-error", theme)
            .cursor_pointer()
            .text_color(theme.warning)
            .child(Icon::empty().path("icons/triangle-alert.svg").size_4())
            .child("Git error")
            .tooltip(move |window, cx| Tooltip::new(message.clone()).build(window, cx))
            .on_click(cx.listener(|workspace, _, window, cx| {
                workspace.show_view(SidebarView::SourceControl, window, cx)
            }))
    }

    fn render_editor_status(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .h_full()
            .gap_1()
            .when_some(self.editor_area.read(cx).active_file(), |bar, file| {
                let position = file.cursor_position();
                bar.child(status_item("cursor", theme).child(format!(
                    "Ln {}, Col {}",
                    position.line + 1,
                    position.character + 1
                )))
                .child(status_item("indent", theme).child("Spaces: 4"))
                .child(status_item("encoding", theme).child("UTF-8"))
                .child(status_item("language", theme).child(file.language_name()))
            })
            .child(
                status_item("notifications", theme)
                    .child(Icon::empty().path("icons/bell.svg").size_4()),
            )
    }
}

fn status_item(id: &'static str, theme: &Theme) -> Stateful<Div> {
    h_flex()
        .id(id)
        .h_full()
        .px_1p5()
        .gap_1()
        .rounded(theme.radius)
        .hover(|style| style.bg(theme.accent).text_color(theme.foreground))
}
