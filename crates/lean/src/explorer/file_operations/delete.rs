use std::path::{Path, PathBuf};

use gpui_kit::{AppContext as _, Context, PromptLevel, SharedString, Window};

use super::DeleteSelected;
use crate::explorer::Explorer;
use crate::explorer::settings::{CONFIRM_DELETE, DELETE_TO, DeleteTo};
use crate::files::file_name;
use crate::plugins::PanelEvent;

const MOVE_TO_TRASH_ANSWER: usize = 0;

#[cfg(windows)]
const TRASH_NAME: &str = "Recycle Bin";
#[cfg(not(windows))]
const TRASH_NAME: &str = "Trash";

impl Explorer {
    pub(super) fn delete_selected(
        &mut self,
        _: &DeleteSelected,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let path = entry.path.clone();
        let is_dir = entry.is_dir;
        let name = file_name(&path);
        let permanently = DELETE_TO.get(cx) == DeleteTo::Permanent;
        match (CONFIRM_DELETE.get(cx), permanently) {
            (false, false) => self.move_to_trash(path, name, window, cx),
            (false, true) => self.delete_permanently(path, name, window, cx),
            (true, false) => self.confirm_move_to_trash(path, name, is_dir, window, cx),
            (true, true) => {
                let question = permanent_delete_question(&name, is_dir);
                let detail = "This can't be undone.".to_string();
                self.confirm_permanent_delete(path, name, question, detail, window, cx);
            }
        }
    }

    fn confirm_move_to_trash(
        &mut self,
        path: PathBuf,
        name: SharedString,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (message, detail) = delete_question(&name, is_dir);
        let move_to_trash = format!("Move to {TRASH_NAME}");
        let answer = window.prompt(
            PromptLevel::Warning,
            &message,
            Some(&detail),
            &[move_to_trash.as_str(), "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |explorer, cx| {
            if !matches!(answer.await, Ok(MOVE_TO_TRASH_ANSWER)) {
                return;
            }
            let update = explorer.update_in(cx, |explorer, window, cx| {
                explorer.move_to_trash(path, name, window, cx)
            });
            if let Err(error) = update {
                log::debug!("explorer dropped before deleting an entry: {error:#}");
            }
        })
        .detach();
    }

    fn move_to_trash(
        &mut self,
        path: PathBuf,
        name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let task = cx.background_spawn({
            let path = path.clone();
            async move { trash::delete(&path) }
        });
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update_in(cx, |explorer, window, cx| match result {
                Ok(()) => explorer.finish_delete(path, cx),
                Err(error) => {
                    log::warn!("failed to move {} to the trash: {error}", path.display());
                    let reason = format!("Can't move \"{name}\" to the {TRASH_NAME}: {error}");
                    let question = format!(
                        "Do you want to permanently delete \"{name}\"? This can't be undone."
                    );
                    explorer.confirm_permanent_delete(path, name, reason, question, window, cx);
                }
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while deleting an entry: {error:#}");
            }
        })
        .detach();
    }

    pub(super) fn finish_delete(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let index = self
            .visible
            .iter()
            .position(|entry| !entry.is_draft && entry.path == path);
        self.expanded.retain(|folder| !folder.starts_with(&path));
        self.children.retain(|folder, _| !folder.starts_with(&path));
        for children in self.children.values_mut() {
            children.retain(|child| child.path != path);
        }
        self.rebuild_visible();
        if let Some(index) = index
            && self
                .selected
                .as_ref()
                .is_some_and(|selected| selected.starts_with(&path))
        {
            let next = index.min(self.visible.len().saturating_sub(1));
            self.selected = self.visible.get(next).map(|entry| entry.path.clone());
        }
        if let Some(folder) = path.parent().map(Path::to_path_buf) {
            self.finish_file_operation(folder, None, cx);
        }
        cx.emit(PanelEvent::PathDeleted(path));
        cx.notify();
    }
}

fn delete_question(name: &str, is_dir: bool) -> (String, String) {
    if is_dir {
        (
            format!("Are you sure you want to delete \"{name}\" and its contents?"),
            format!("You can restore this folder from the {TRASH_NAME}."),
        )
    } else {
        (
            format!("Are you sure you want to delete \"{name}\"?"),
            format!("You can restore this file from the {TRASH_NAME}."),
        )
    }
}

fn permanent_delete_question(name: &str, is_dir: bool) -> String {
    if is_dir {
        format!("Are you sure you want to permanently delete \"{name}\" and its contents?")
    } else {
        format!("Are you sure you want to permanently delete \"{name}\"?")
    }
}
