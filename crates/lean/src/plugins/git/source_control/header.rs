use gpui_kit::component::{ActiveTheme as _, Icon, h_flex};
use gpui_kit::*;

use super::SourceControl;
use crate::plugins::git::settings::ViewMode;
use crate::ui::icon_button;

impl SourceControl {
    pub(super) fn render_header(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        let branch = self
            .git
            .read(cx)
            .state()
            .map(|state| state.branch.clone())
            .unwrap_or_default();
        let (view_icon, view_tooltip) = match self.view_mode {
            ViewMode::Tree => ("icons/list.svg", "View as List"),
            ViewMode::List => ("icons/list-tree.svg", "View as Tree"),
        };
        h_flex()
            .h(px(30.))
            .flex_shrink_0()
            .pl_2()
            .pr_1()
            .gap_1p5()
            .child(
                Icon::empty()
                    .path("icons/git-branch.svg")
                    .size_4()
                    .flex_shrink_0()
                    .text_color(theme.muted_foreground),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(branch),
            )
            .child(
                icon_button("toggle-view", view_icon, view_tooltip).on_click(
                    cx.listener(|source_control, _, _, cx| source_control.toggle_view_mode(cx)),
                ),
            )
            .child(
                icon_button("refresh", "icons/refresh-cw.svg", "Refresh")
                    .loading(self.is_busy())
                    .on_click(cx.listener(|source_control, _, _, cx| source_control.refresh(cx))),
            )
    }
}
