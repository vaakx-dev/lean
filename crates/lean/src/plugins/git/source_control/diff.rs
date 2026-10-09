use gpui_kit::{Context, Window};

use super::SourceControl;
use super::changes::{ChangeGroup, ChangedFile};
use crate::plugins::git::diff::DiffBase;

impl SourceControl {
    pub(super) fn open_diff(
        &mut self,
        file: &ChangedFile,
        group: ChangeGroup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base = match group {
            ChangeGroup::Staged => DiffBase::Staged,
            ChangeGroup::Unstaged => DiffBase::Index,
        };
        let path = file.path.clone();
        self.git
            .update(cx, |git, cx| git.open_diff(path, base, window, cx));
    }
}
