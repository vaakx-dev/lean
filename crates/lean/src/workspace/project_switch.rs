use std::path::PathBuf;

use gpui_kit::*;

use super::Workspace;
use super::close_confirmation::CloseIntent;

impl Workspace {
    pub(super) fn switch_project(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let confirmation = self.confirm_close(CloseIntent::SwitchProject, window, cx);
        cx.spawn_in(window, async move |workspace, cx| {
            if !confirmation.await {
                return;
            }
            let update = workspace.update_in(cx, |workspace, window, cx| {
                workspace.set_project_root(path, window, cx)
            });
            if let Err(error) = update {
                log::debug!("workspace dropped while opening a folder: {error:#}");
            }
        })
        .detach();
    }
}
