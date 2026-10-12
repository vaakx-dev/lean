use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::{Context, Window};

use super::{DeleteSelected, report_failure};
use crate::explorer::Explorer;
use crate::explorer::settings::{CONFIRM_DELETE, DELETE_TO, DeleteTo};
use crate::files::file_name;
use crate::plugins::PanelEvent;

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
        let path = entry.path.to_path_buf();
        let is_dir = entry.is_dir;
        let name = file_name(&path);
        let permanently = DELETE_TO.get(cx) == DeleteTo::Permanent;
        match (CONFIRM_DELETE.get(cx), permanently) {
            (false, false) => self.move_to_trash(path, window, cx),
            (false, true) => self.delete_permanently(path, window, cx),
            (true, false) => {
                let (message, detail) = delete_question(&name, is_dir);
                let button = format!("Move to {TRASH_NAME}");
                self.confirm(
                    &message,
                    &detail,
                    &button,
                    window,
                    cx,
                    move |explorer, window, cx| explorer.move_to_trash(path, window, cx),
                );
            }
            (true, true) => {
                let message = permanent_delete_question(&name, is_dir);
                self.confirm_permanent_delete(path, &message, "This can't be undone.", window, cx);
            }
        }
    }

    fn confirm_permanent_delete(
        &mut self,
        path: PathBuf,
        message: &str,
        detail: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm(
            message,
            detail,
            "Delete Permanently",
            window,
            cx,
            move |explorer, window, cx| explorer.delete_permanently(path, window, cx),
        );
    }

    fn move_to_trash(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        let target = path.clone();
        self.run_disk_op(
            move || trash::delete(&target),
            window,
            cx,
            move |explorer, result, window, cx| match result {
                Ok(()) => explorer.finish_delete(path, window, cx),
                Err(error) => {
                    log::warn!("failed to move {} to the trash: {error}", path.display());
                    let name = file_name(&path);
                    let reason = format!("Can't move \"{name}\" to the {TRASH_NAME}: {error}");
                    let question = format!(
                        "Do you want to permanently delete \"{name}\"? This can't be undone."
                    );
                    explorer.confirm_permanent_delete(path, &reason, &question, window, cx);
                }
            },
        );
    }

    fn delete_permanently(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        let target = path.clone();
        self.run_disk_op(
            move || remove_from_disk(&target),
            window,
            cx,
            move |explorer, result, window, cx| match result {
                Ok(()) => explorer.finish_delete(path, window, cx),
                Err(error) => report_failure("delete", &path, error, window, cx),
            },
        );
    }

    fn finish_delete(&mut self, path: PathBuf, window: &Window, cx: &mut Context<Self>) {
        let index = self
            .visible
            .iter()
            .position(|entry| !entry.is_draft && *entry.path == *path);
        self.forget_subtrees(&[&path]);
        if let Some(parent) = path.parent()
            && let Some(siblings) = self.children.get_mut(parent)
        {
            siblings.retain(|child| *child.path != *path);
        }
        self.rebuild_visible();
        if let Some(index) = index
            && self
                .selected
                .as_ref()
                .is_some_and(|selected| selected.starts_with(&path))
        {
            let next = index.min(self.visible.len().saturating_sub(1));
            self.selected = self.visible.get(next).map(|entry| entry.path.to_path_buf());
        }
        if let Some(folder) = path.parent().map(Path::to_path_buf) {
            self.finish_file_operation(folder, None, window, cx);
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

fn remove_from_disk(path: &Path) -> io::Result<()> {
    if path.symlink_metadata()?.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}
