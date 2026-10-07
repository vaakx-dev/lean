use gpui_kit::component::{ActiveTheme as _, Icon, Theme, WindowExt as _, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use super::sidebar::SidebarView;

pub(super) const ACTIVITY_BAR_WIDTH: Pixels = px(52.);

impl Workspace {
    pub(super) fn render_activity_bar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let change_count = self
            .project
            .read(cx)
            .git()
            .map_or(0, |git| git.statuses.len());
        v_flex()
            .w(ACTIVITY_BAR_WIDTH)
            .flex_shrink_0()
            .h_full()
            .items_center()
            .justify_between()
            .pb_1()
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_activity_item(SidebarView::Explorer, "icons/files.svg", cx))
                    .child(self.render_activity_item(SidebarView::Search, "icons/search.svg", cx))
                    .child(
                        self.render_activity_item(
                            SidebarView::SourceControl,
                            "icons/git-branch.svg",
                            cx,
                        )
                        .when(change_count > 0, |item| {
                            item.child(change_badge(change_count, theme))
                        }),
                    ),
            )
            .child(
                activity_button("settings", "icons/settings.svg", theme).on_click(
                    |_, window, cx| window.push_notification("Settings are not available yet.", cx),
                ),
            )
    }

    fn render_activity_item(
        &self,
        view: SidebarView,
        icon: &'static str,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let is_active = self.sidebar_visible && self.sidebar_view == view;
        activity_button(view.title(), icon, theme)
            .relative()
            .when(is_active, |item| {
                item.bg(theme.accent).text_color(theme.foreground)
            })
            .on_click(
                cx.listener(move |workspace, _, window, cx| {
                    workspace.toggle_view(view, window, cx)
                }),
            )
    }
}

fn activity_button(id: &'static str, icon: &'static str, theme: &Theme) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(40.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(theme.radius)
        .cursor_pointer()
        .text_color(theme.muted_foreground)
        .hover(|style| style.text_color(theme.foreground))
        .child(Icon::empty().path(icon).size(px(22.)))
}

fn change_badge(count: usize, theme: &Theme) -> Div {
    let text = if count > 999 {
        "999+".to_string()
    } else {
        count.to_string()
    };
    div()
        .absolute()
        .bottom(px(1.))
        .right(px(-2.))
        .min_w(px(16.))
        .h(px(16.))
        .px_1()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme.primary)
        .text_color(theme.primary_foreground)
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(text)
}
