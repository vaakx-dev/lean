use gpui_kit::Context;

use super::GitProject;
use crate::plugins::git::settings::history_page_size;

impl GitProject {
    pub(crate) fn load_more_history(&mut self, cx: &mut Context<Self>) {
        self.history_limit += history_page_size(cx);
        self.refresh(cx);
    }
}
