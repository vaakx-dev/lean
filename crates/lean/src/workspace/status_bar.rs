use gpui_kit::component::{ActiveTheme as _, Theme, h_flex};
use gpui_kit::*;

use super::Workspace;

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
            .child(
                h_flex()
                    .h_full()
                    .gap_1()
                    .children(self.status_items.iter().cloned()),
            )
            .child(
                h_flex()
                    .h_full()
                    .gap_1()
                    .children(self.context.editor_area.read(cx).status_items(cx)),
            )
    }
}

pub(crate) fn status_item(id: &'static str, theme: &Theme) -> Stateful<Div> {
    h_flex()
        .id(id)
        .h_full()
        .px_1p5()
        .gap_1()
        .rounded(theme.radius)
        .hover(|style| style.bg(theme.accent).text_color(theme.foreground))
}
