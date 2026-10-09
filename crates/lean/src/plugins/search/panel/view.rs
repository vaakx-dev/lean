use gpui_kit::component::v_flex;
use gpui_kit::*;

use super::SearchPanel;

impl Render for SearchPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .track_focus(&self.focus_handle)
            .child(self.render_query_bar(cx))
            .children(self.render_summary(cx))
            .child(
                uniform_list(
                    "search-results",
                    self.rows.len(),
                    cx.processor(Self::render_rows),
                )
                .track_scroll(&self.scroll_handle)
                .flex_1()
                .px_1()
                .pb_2(),
            )
    }
}
