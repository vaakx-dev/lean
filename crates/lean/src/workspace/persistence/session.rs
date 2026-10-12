use std::path::{Path, PathBuf};

use gpui_kit::*;

use super::session_store::{self, Session};
use crate::workspace::Workspace;
use crate::workspace::settings::RESTORE_SESSION;

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
        let session = if RESTORE_SESSION.get(cx) {
            session_store::session(&root, cx)
        } else {
            None
        };
        if session.is_none() && extra.is_none() {
            return;
        }
        let existing = cx.background_spawn(async move { existing_files(session) });
        cx.spawn_in(window, async move |workspace, cx| {
            let mut paths = existing.await;
            paths.extend(extra);
            for path in paths {
                let opened = workspace.update_in(cx, |workspace, window, cx| {
                    (workspace.project_root(cx) == root).then(|| {
                        workspace
                            .context
                            .editor_area
                            .update(cx, |area, cx| area.open_pinned(path, window, cx))
                    })
                });
                match opened {
                    Ok(Some(opened)) => opened.await,
                    _ => return,
                }
            }
        })
        .detach();
    }
}

fn existing_files(session: Option<Session>) -> Vec<PathBuf> {
    session
        .into_iter()
        .flat_map(|session| session.files.into_iter().chain(session.active))
        .filter(|path| path.is_file())
        .collect()
}
