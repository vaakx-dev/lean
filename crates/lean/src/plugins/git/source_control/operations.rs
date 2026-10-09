use std::path::{Path, PathBuf};

use anyhow::Result;
use futures::FutureExt as _;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::SourceControl;
use super::changes::ChangeGroup;
use crate::plugins::git::repo as git;

impl SourceControl {
    pub(super) fn stage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() {
            return;
        }
        self.run_git_operation(
            "stage changes",
            move |root| git::stage(root, &paths),
            window,
            cx,
        );
    }

    pub(super) fn unstage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() {
            return;
        }
        self.run_git_operation(
            "unstage changes",
            move |root| git::unstage(root, &paths),
            window,
            cx,
        );
    }

    pub(super) fn discard(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() {
            return;
        }
        self.run_git_operation(
            "discard changes",
            move |root| git::discard(root, &paths),
            window,
            cx,
        );
    }

    pub(super) fn stage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = self.changes.paths(ChangeGroup::Unstaged);
        self.stage(paths, window, cx);
    }

    pub(super) fn unstage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = self.changes.paths(ChangeGroup::Staged);
        self.unstage(paths, window, cx);
    }

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.git.update(cx, |git, cx| git.refresh(cx));
    }

    pub(super) fn is_busy(&self) -> bool {
        self.pending_git_operations > 0
    }

    pub(super) fn run_git_operation(
        &mut self,
        description: &'static str,
        operation: impl FnOnce(&Path) -> Result<()> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = self.git.read(cx).root(cx);
        let previous = self.git_operation.clone();
        self.pending_git_operations += 1;
        cx.notify();
        let task = cx.spawn_in(window, async move |source_control, cx| {
            previous.await;
            let result = cx
                .background_executor()
                .spawn(async move { operation(&root) })
                .await;
            let update = source_control.update_in(cx, |source_control, window, cx| {
                source_control.pending_git_operations -= 1;
                if let Err(error) = &result {
                    log::error!("failed to {description}: {error:#}");
                    window.push_notification(
                        Notification::error(format!("Failed to {description}: {error:#}")),
                        cx,
                    );
                }
                source_control.refresh(cx);
                cx.notify();
            });
            if let Err(error) = &update {
                log::debug!("source control closed before {description} finished: {error:#}");
            }
            result.is_ok() && update.is_ok()
        });
        self.git_operation = task.shared();
    }
}
