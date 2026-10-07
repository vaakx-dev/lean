use gpui_kit::component::v_flex;
use gpui_kit::*;

use super::SourceControl;
use crate::placeholder::placeholder;

impl Render for SourceControl {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let project = self.project.read(cx);
        if let Some(error) = project.git_error().cloned() {
            return self.render_git_error(error, cx);
        }
        if project.git().is_none() {
            return placeholder("This folder is not a git repository.", cx);
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
