use std::path::{Path, PathBuf};

use gpui_kit::*;

use super::GitProject;
use crate::plugins::git::diff::{DiffBase, DiffView};

const DIFF_ICON: &str = "icons/file-diff.svg";

impl GitProject {
    pub(crate) fn open_diff(
        &mut self,
        path: PathBuf,
        base: DiffBase,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(workdir) = self.state.as_ref().map(|state| state.workdir.clone()) else {
            return;
        };
        let key = PathBuf::from(format!("git-diff:{}:{}", base.key(), path.display()));
        if self
            .editor_area
            .update(cx, |area, cx| area.show_view(&key, window, cx))
        {
            return;
        }
        let view = DiffView::open(workdir, path, base, window, cx);
        let title = view.read(cx).title();
        self.diff_views.retain(|view| view.upgrade().is_some());
        self.diff_views.push(view.downgrade());
        self.editor_area.update(cx, |area, cx| {
            area.open_view(key, title, DIFF_ICON.into(), view.into(), window, cx)
        });
    }

    pub(super) fn refresh_diff_views(
        &mut self,
        shows: impl Fn(&Path) -> bool,
        cx: &mut Context<Self>,
    ) {
        self.diff_views.retain(|view| view.upgrade().is_some());
        for view in self.diff_views.iter().filter_map(WeakEntity::upgrade) {
            if shows(view.read(cx).path()) {
                view.update(cx, |view, cx| view.refresh(cx));
            }
        }
    }
}
