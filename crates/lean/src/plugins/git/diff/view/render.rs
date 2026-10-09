use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::*;

use super::DiffView;

impl Render for DiffView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx: &Context<Self> = cx;
        let theme = cx.theme();
        let root = div()
            .id("diff-view")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme.mono_font_family.clone())
            .text_size(theme.mono_font_size);
        if let Some(error) = &self.error {
            return root.child(render_message(error.clone(), cx));
        }
        let Some(document) = &self.document else {
            return root.child(render_message("Loading diff…".into(), cx));
        };
        if let Some(notice) = document.notice {
            return root.child(render_message(notice.into(), cx));
        }
        root.child(
            div()
                .relative()
                .size_full()
                .child(
                    uniform_list(
                        "diff-rows",
                        document.rows.len(),
                        cx.processor(Self::render_rows),
                    )
                    .track_scroll(&self.scroll_handle)
                    .size_full()
                    .pb_4(),
                )
                .vertical_scrollbar(&self.scroll_handle),
        )
    }
}

fn render_message(message: SharedString, cx: &App) -> Div {
    div()
        .p_4()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(message)
}
