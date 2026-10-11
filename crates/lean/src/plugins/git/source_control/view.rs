use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::*;

use super::SourceControl;

impl Render for SourceControl {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .child(self.render_content(cx))
    }
}

impl SourceControl {
    fn render_content(&self, cx: &Context<Self>) -> AnyElement {
        let git = self.git.read(cx);
        if let Some(error) = git.error().cloned() {
            return self.render_git_error(error, cx);
        }
        if git.state().is_none() {
            return div()
                .size_full()
                .p_4()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("This folder is not a git repository.")
                .into_any_element();
        }
        v_flex()
            .size_full()
            .child(self.render_header(cx))
            .child(self.render_commit_box(cx))
            .child(
                uniform_list(
                    "source-control-rows",
                    self.rows.len(),
                    cx.processor(Self::render_rows),
                )
                .track_scroll(&self.scroll_handle)
                .flex_1()
                .px_1()
                .pb_2(),
            )
            .into_any_element()
    }
}
