use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::SourceControl;
use super::changes::{ChangeGroup, paths_of};
use crate::plugins::git::repo as git;

impl SourceControl {
    pub(super) fn stage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_path_operation("stage changes", git::stage, paths, window, cx);
    }

    pub(super) fn unstage(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_path_operation("unstage changes", git::unstage, paths, window, cx);
    }

    pub(super) fn discard(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.run_path_operation("discard changes", git::discard, paths, window, cx);
    }

    pub(super) fn stage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = paths_of(self.changes.files(ChangeGroup::Unstaged));
        self.stage(paths, window, cx);
    }

    pub(super) fn unstage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = paths_of(self.changes.files(ChangeGroup::Staged));
        self.unstage(paths, window, cx);
    }

    pub(super) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.git.update(cx, |git, cx| git.refresh(cx));
    }

    pub(super) fn is_busy(&self) -> bool {
        self.operation.is_some()
    }

    fn run_path_operation(
        &mut self,
        description: &'static str,
        operation: fn(&Path, &[PathBuf]) -> Result<()>,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if paths.is_empty() {
            return;
        }
        self.run_git_operation(
            description,
            move |root| operation(root, &paths),
            |_, _, _| {},
            window,
            cx,
        );
    }

    pub(super) fn run_git_operation(
        &mut self,
        description: &'static str,
        operation: impl FnOnce(&Path) -> Result<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_busy() {
            return;
        }
        let root = self.git.read(cx).root(cx);
        self.operation = Some(cx.spawn_in(window, async move |source_control, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { operation(&root) })
                .await;
            source_control
                .update_in(cx, |source_control, window, cx| {
                    source_control.operation = None;
                    match result {
                        Ok(()) => on_success(source_control, window, cx),
                        Err(error) => {
                            log::error!("failed to {description}: {error:#}");
                            window.push_notification(
                                Notification::error(format!("Failed to {description}: {error:#}")),
                                cx,
                            );
                        }
                    }
                    source_control.refresh(cx);
                    cx.notify();
                })
                .ok();
        }));
        cx.notify();
    }
}
