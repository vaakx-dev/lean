use std::path::Path;
use std::time::Instant;

use gpui_kit::Context;

use super::GitProject;
use crate::watcher::FsChanges;

impl GitProject {
    pub(super) fn files_changed(&mut self, changes: &FsChanges, cx: &mut Context<Self>) {
        let affects_git = changes.metadata || changes.touches_files();
        if affects_git && !self.read_covers(changes) {
            self.schedule_refresh(cx);
        }
        if changes.metadata || changes.rescan {
            self.git_metadata_changed(changes.observed_at, cx);
        } else {
            self.refresh_changed_paths(changes, cx);
        }
    }

    pub(super) fn refresh_views(&mut self, cx: &mut Context<Self>) {
        self.refresh.views_refreshed = Some(Instant::now());
        self.refresh_diff_views(|_| true, cx);
        self.refresh_gutter(cx);
    }

    fn git_metadata_changed(&mut self, observed_at: Option<Instant>, cx: &mut Context<Self>) {
        let views_are_fresh = observed_at
            .zip(self.refresh.views_refreshed)
            .is_some_and(|(observed, refreshed)| refreshed > observed);
        if views_are_fresh {
            return;
        }
        if self.refresh.is_pending() {
            self.refresh.views_stale = true;
        } else {
            self.refresh_views(cx);
        }
    }

    fn refresh_changed_paths(&mut self, changes: &FsChanges, cx: &mut Context<Self>) {
        let touched = |path: &Path| is_touched(changes, path);
        self.refresh_diff_views(touched, cx);
        self.refresh_gutter_where(touched, cx);
    }
}

fn is_touched(changes: &FsChanges, path: &Path) -> bool {
    path.ancestors()
        .any(|folder| changes.paths.contains(folder) || changes.ignored.contains(folder))
        || changes
            .renames
            .iter()
            .any(|(from, to)| from == path || to == path)
}
