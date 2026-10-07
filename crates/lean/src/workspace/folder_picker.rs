use std::path::PathBuf;

use anyhow::Result;
use futures::channel::oneshot;
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::OpenFolder;

const DISCARD_ANSWER: usize = 0;

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
            let confirmation = workspace.update_in(cx, |workspace, window, cx| {
                workspace.confirm_discarding_unsaved_files(window, cx)
            });
            let confirmation = match confirmation {
                Ok(confirmation) => confirmation,
                Err(error) => {
                    log::debug!("workspace dropped while picking a folder: {error:#}");
                    return;
                }
            };
            if let Some(answer) = confirmation {
                match answer.await {
                    Ok(DISCARD_ANSWER) => {}
                    Ok(_) => return,
                    Err(error) => {
                        log::debug!("the discard prompt was dismissed: {error}");
                        return;
                    }
                }
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

    fn confirm_discarding_unsaved_files(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<oneshot::Receiver<usize>> {
        if !self.editor_area.read(cx).has_unsaved_files() {
            return None;
        }
        Some(window.prompt(
            PromptLevel::Warning,
            "Do you want to discard your unsaved changes?",
            Some("Opening a folder closes all open files. Unsaved changes will be lost."),
            &[
                PromptButton::new("Discard Changes"),
                PromptButton::cancel("Cancel"),
            ],
            cx,
        ))
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
