use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::SourceControl;
use super::changes::ChangeGroup;
use crate::git;

impl SourceControl {
    pub(super) fn stage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_git_operation(
            "stage changes",
            move |root| git::stage(root, &paths),
            window,
            cx,
        )
        .detach();
    }

    pub(super) fn unstage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_git_operation(
            "unstage changes",
            move |root| git::unstage(root, &paths),
            window,
            cx,
        )
        .detach();
    }

    pub(super) fn discard(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_git_operation(
            "discard changes",
            move |root| git::discard(root, &paths),
            window,
            cx,
        )
        .detach();
    }

    pub(super) fn stage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.run_git_operation("stage all changes", git::stage_all, window, cx)
            .detach();
    }

    pub(super) fn unstage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = self.changes.paths(ChangeGroup::Staged);
        self.unstage(paths, window, cx);
    }

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.project
            .update(cx, |project, cx| project.refresh_git(cx));
    }

    pub(super) fn run_git_operation(
        &mut self,
        description: &'static str,
        operation: impl FnOnce(&Path) -> Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let root = self.project.read(cx).root().to_path_buf();
        let task = cx.background_spawn(async move { operation(&root) });
        cx.spawn_in(window, async move |source_control, cx| {
            let result = task.await;
            let update = source_control.update_in(cx, |source_control, window, cx| {
                if let Err(error) = &result {
                    log::error!("failed to {description}: {error:#}");
                    window.push_notification(
                        Notification::error(format!("Failed to {description}: {error:#}")),
                        cx,
                    );
                }
                source_control.refresh(cx);
            });
            if let Err(error) = &update {
                log::debug!("source control closed before {description} finished: {error:#}");
            }
            result.is_ok() && update.is_ok()
        })
    }
}
