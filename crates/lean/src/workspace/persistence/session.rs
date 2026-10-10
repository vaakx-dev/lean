use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::*;

use super::session_store::{self, Session};
use crate::workspace::Workspace;
use crate::workspace::settings::RESTORE_SESSION;

const OPEN_POLL_INTERVAL: Duration = Duration::from_millis(25);
const OPEN_POLL_LIMIT: usize = 80;

impl Workspace {
    pub(in crate::workspace) fn record_session(&self, cx: &mut App) {
        if !RESTORE_SESSION.get(cx) {
            return;
        }
        let area = self.context.editor_area.read(cx);
        let session = Session {
            files: area
                .open_file_views()
                .map(|(path, _)| path.to_path_buf())
                .collect(),
            active: area.active_path().map(Path::to_path_buf),
        };
        let root = self.project_root(cx);
        session_store::record(&root, session, cx);
    }

    pub(in crate::workspace) fn reopen_files(
        &self,
        extra: Option<PathBuf>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.project_root(cx);
        let mut paths = Vec::new();
        if RESTORE_SESSION.get(cx)
            && let Some(session) = session_store::session(&root, cx)
        {
            paths.extend(session.files.into_iter().filter(|path| path.is_file()));
            paths.extend(session.active.filter(|path| path.is_file()));
        }
        paths.extend(extra);
        if paths.is_empty() {
            return;
        }
        cx.spawn_in(window, async move |workspace, cx| {
            for path in paths {
                let opened = workspace.update_in(cx, |workspace, window, cx| {
                    let same_project = workspace.project_root(cx) == root;
                    if same_project {
                        workspace
                            .context
                            .editor_area
                            .update(cx, |area, cx| area.open_pinned(path.clone(), window, cx));
                    }
                    same_project
                });
                if !matches!(opened, Ok(true)) {
                    return;
                }
                wait_until_open(&workspace, &path, cx).await;
            }
        })
        .detach();
    }
}

async fn wait_until_open(
    workspace: &WeakEntity<Workspace>,
    path: &Path,
    cx: &mut AsyncWindowContext,
) {
    for _ in 0..OPEN_POLL_LIMIT {
        let is_open = workspace
            .read_with(cx, |workspace, cx| {
                workspace
                    .context
                    .editor_area
                    .read(cx)
                    .open_file_views()
                    .any(|(open, _)| open == path)
            })
            .unwrap_or(true);
        if is_open {
            return;
        }
        cx.background_executor().timer(OPEN_POLL_INTERVAL).await;
    }
}
