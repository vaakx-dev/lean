use std::path::PathBuf;

use anyhow::Result;
use futures::channel::oneshot;
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::OpenFolder;

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
            let update = workspace.update_in(cx, |workspace, window, cx| {
                workspace.switch_project(path, window, cx)
            });
            if let Err(error) = update {
                log::debug!("workspace dropped while picking a folder: {error:#}");
            }
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
