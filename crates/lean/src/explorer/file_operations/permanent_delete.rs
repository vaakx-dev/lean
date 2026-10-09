use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AppContext as _, Context, PromptLevel, SharedString, Window};

use crate::explorer::Explorer;

const DELETE_PERMANENTLY_ANSWER: usize = 0;

impl Explorer {
    pub(super) fn offer_permanent_delete(
        &mut self,
        path: PathBuf,
        name: SharedString,
        reason: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let question =
            format!("Do you want to permanently delete \"{name}\"? This can't be undone.");
        let answer = window.prompt(
            PromptLevel::Warning,
            &reason,
            Some(&question),
            &["Delete Permanently", "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |explorer, cx| {
            if !matches!(answer.await, Ok(DELETE_PERMANENTLY_ANSWER)) {
                return;
            }
            let update = explorer.update_in(cx, |explorer, window, cx| {
                explorer.delete_permanently(path, name, window, cx)
            });
            if let Err(error) = update {
                log::debug!("explorer dropped before deleting an entry: {error:#}");
            }
        })
        .detach();
    }

    fn delete_permanently(
        &mut self,
        path: PathBuf,
        name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let task = cx.background_spawn({
            let path = path.clone();
            async move { remove_from_disk(&path) }
        });
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update_in(cx, |explorer, window, cx| match result {
                Ok(()) => explorer.finish_delete(path, cx),
                Err(error) => {
                    log::warn!("failed to delete {}: {error}", path.display());
                    window.push_notification(format!("Can't delete \"{name}\": {error}"), cx);
                }
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while deleting an entry: {error:#}");
            }
        })
        .detach();
    }
}

fn remove_from_disk(path: &Path) -> io::Result<()> {
    if path.symlink_metadata()?.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}
