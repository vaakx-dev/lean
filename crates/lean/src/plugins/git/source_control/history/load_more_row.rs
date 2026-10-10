use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use crate::plugins::git::source_control::SourceControl;
use crate::plugins::git::source_control::rows::clickable_row;

impl SourceControl {
    pub(in crate::plugins::git::source_control) fn render_load_more_row(
        &self,
        index: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        clickable_row(index, theme)
            .pl_2()
            .text_sm()
            .text_color(theme.muted_foreground)
            .hover(|style| style.bg(theme.list_hover).text_color(theme.foreground))
            .child("Load more commits…")
            .on_click(cx.listener(|source_control, _, _, cx| {
                source_control
                    .git
                    .update(cx, |git, cx| git.load_more_history(cx))
            }))
            .into_any_element()
    }
}
