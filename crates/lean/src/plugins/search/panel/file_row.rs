use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SearchPanel;
use super::rows::{chevron, clickable_row};
use crate::files::file_icon;

impl SearchPanel {
    pub(super) fn render_file_row(
        &self,
        index: usize,
        file_index: usize,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        let theme = cx.theme();
        let file = self.results.files.get(file_index)?;
        let expanded = !self.collapsed.contains(&file_index);
        let row = clickable_row(index, theme)
            .pl_1()
            .gap_1()
            .child(chevron(expanded, theme))
            .child(file_icon(&file.name).flex_shrink_0())
            .child(
                div()
                    .flex_shrink_1()
                    .min_w_0()
                    .truncate()
                    .child(file.name.clone()),
            )
            .when(!file.folder.is_empty(), |row| {
                row.child(
                    div()
                        .flex_shrink_1()
                        .min_w(px(24.))
                        .truncate()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(file.folder.clone()),
                )
            })
            .child(div().flex_1())
            .child(count_badge(file.match_count, theme))
            .on_click(cx.listener(move |panel, _, _, cx| panel.toggle_file(file_index, cx)));
        Some(row.into_any_element())
    }
}

fn count_badge(count: usize, theme: &Theme) -> Div {
    div()
        .flex_shrink_0()
        .min_w(px(18.))
        .px_1p5()
        .flex()
        .justify_center()
        .rounded_full()
        .bg(theme.accent)
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(count.to_string())
}
