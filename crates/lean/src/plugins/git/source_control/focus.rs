use gpui_kit::*;

use super::SourceControl;

impl SourceControl {
    pub(super) fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shows_changes(cx) {
            self.commit_message
                .update(cx, |input, cx| input.focus(window, cx));
        } else {
            window.focus(&self.focus_handle, cx);
        }
    }

    pub(super) fn shows_changes(&self, cx: &App) -> bool {
        let git = self.git.read(cx);
        git.error().is_none() && git.state().is_some()
    }

    pub(super) fn keep_focus_rendered(&self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.shows_changes(cx)
            && self
                .commit_message
                .focus_handle(cx)
                .contains_focused(window, cx)
        {
            window.focus(&self.focus_handle, cx);
        }
    }
}
