use std::path::PathBuf;

use gpui_kit::component::WindowExt as _;
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::{ClearRecentProjects, OpenRecent, OpenRecentProject};
use crate::{app, recent_projects};

impl Workspace {
    pub(super) fn open_recent(
        &mut self,
        _: &OpenRecent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let workspace = cx.weak_entity();
        recent_projects::open_picker(window, cx, move |path, window, cx| {
            let update = workspace.update(cx, |workspace, cx| {
                workspace.open_recent_path(path, window, cx)
            });
            if let Err(error) = update {
                log::debug!("workspace dropped while picking a recent folder: {error:#}");
            }
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
        let root = self.context.project.read(cx).root().to_path_buf();
        if recent_projects::record_opened(root, cx) {
            self.refresh_app_menus(cx);
        }
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

    pub(super) fn refresh_app_menus(&self, cx: &mut Context<Self>) {
        app::set_menus(cx);
        self.app_menu_bar
            .update(cx, |menu_bar, cx| menu_bar.reload(cx));
    }
}
