use gpui_kit::component::badge::Badge;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, Theme, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use super::panels::SidebarPanel;

pub(super) const ACTIVITY_BAR_WIDTH: Pixels = px(52.);
const BADGE_MAX: usize = 999;

impl Workspace {
    pub(super) fn render_activity_bar(&self, cx: &Context<Self>) -> Div {
        v_flex()
            .w(ACTIVITY_BAR_WIDTH)
            .flex_shrink_0()
            .h_full()
            .items_center()
            .justify_between()
            .pb_1()
            .child(
                v_flex().gap_1().children(
                    self.panels
                        .iter()
                        .enumerate()
                        .map(|(index, panel)| self.render_activity_item(index, panel, cx)),
                ),
            )
            .child(self.render_quick_settings(cx))
    }

    fn render_activity_item(
        &self,
        index: usize,
        panel: &SidebarPanel,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let is_active = self.sidebar_visible && self.active_panel == index;
        let badge = panel.view.badge(cx);
        activity_button(panel.panel.id, panel.panel.title, theme)
            .when(is_active, |item| {
                item.bg(theme.accent).text_color(theme.foreground)
            })
            .child(
                Badge::new()
                    .count(badge)
                    .max(BADGE_MAX)
                    .color(theme.primary)
                    .child(Icon::empty().path(panel.panel.icon).size(px(22.))),
            )
            .on_click(cx.listener(move |workspace, _, window, cx| {
                workspace.toggle_panel(index, window, cx)
            }))
    }
}

fn activity_button(id: &'static str, title: &'static str, theme: &Theme) -> Stateful<Div> {
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
        .tooltip(move |window, cx| Tooltip::new(title).build(window, cx))
}
