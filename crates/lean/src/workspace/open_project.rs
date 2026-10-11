use std::path::PathBuf;

use anyhow::Result;
use futures::channel::oneshot;
use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::Workspace;
use super::close_confirmation::CloseIntent;
use crate::app::actions::{ClearRecentProjects, OpenFolder, OpenRecent, OpenRecentProject};
use crate::{app, recent_projects};

impl Workspace {
    pub(super) fn open_folder(
        &mut self,
        _: &OpenFolder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Folder".into()),
        });
        cx.spawn_in(window, async move |workspace, cx| {
            let Some(path) = picked_folder(paths.await) else {
                return;
            };
            workspace
                .update_in(cx, |workspace, window, cx| {
                    workspace.switch_project(path, window, cx)
                })
                .ok();
        })
        .detach();
    }

    pub(super) fn open_recent(
        &mut self,
        _: &OpenRecent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let workspace = cx.weak_entity();
        recent_projects::open_picker(window, cx, move |path, window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    workspace.open_recent_path(path, window, cx)
                })
                .ok();
        });
    }

    pub(super) fn open_recent_project(
        &mut self,
        action: &OpenRecentProject,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_recent_path(action.path.clone(), window, cx);
    }

    pub(super) fn clear_recent_projects(
        &mut self,
        _: &ClearRecentProjects,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        recent_projects::clear(cx);
        self.refresh_app_menus(cx);
    }

    pub(super) fn remember_project(&self, cx: &mut Context<Self>) {
        if recent_projects::record_opened(self.project_root(cx), cx) {
            self.refresh_app_menus(cx);
        }
    }

    pub(super) fn refresh_app_menus(&self, cx: &mut Context<Self>) {
        app::set_menus(cx);
        self.app_menu_bar
            .update(cx, |menu_bar, cx| menu_bar.reload(cx));
    }

    fn open_recent_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if !path.is_dir() {
            recent_projects::forget(&path, cx);
            self.refresh_app_menus(cx);
            window.push_notification(
                format!("{} no longer exists.", recent_projects::display_path(&path)),
                cx,
            );
            return;
        }
        self.switch_project(path, window, cx);
    }

    fn switch_project(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let confirmation = self.confirm_close(CloseIntent::SwitchProject, window, cx);
        cx.spawn_in(window, async move |workspace, cx| {
            if !confirmation.await {
                return;
            }
            workspace
                .update_in(cx, |workspace, window, cx| {
                    workspace.set_project_root(path, window, cx)
                })
                .ok();
        })
        .detach();
    }
}

fn picked_folder(
    paths: Result<Result<Option<Vec<PathBuf>>>, oneshot::Canceled>,
) -> Option<PathBuf> {
    match paths {
        Ok(Ok(paths)) => paths?.into_iter().next(),
        Ok(Err(error)) => {
            log::error!("failed to open the folder picker: {error:#}");
            None
        }
        Err(error) => {
            log::error!("the folder picker was closed unexpectedly: {error}");
            None
        }
    }
}
