mod copy_path;
mod create;
mod delete;
mod edit_confirmation;
mod edit_field;
mod inline_edit;
mod name_validation;
mod permanent_delete;
mod rename;
mod reveal;

use std::path::PathBuf;

use gpui_kit::{Context, Div, InteractiveElement as _, Stateful, actions};

use super::Explorer;

pub(super) use inline_edit::InlineEdit;

actions!(
    explorer,
    [
        NewFile,
        NewFolder,
        RenameSelected,
        DeleteSelected,
        CopyPath,
        CopyRelativePath,
        RevealInFileManager
    ]
);

pub(super) fn bind_file_operations(
    element: Stateful<Div>,
    cx: &Context<Explorer>,
) -> Stateful<Div> {
    element
        .on_action(cx.listener(Explorer::new_file))
        .on_action(cx.listener(Explorer::new_folder))
        .on_action(cx.listener(Explorer::rename_selected))
        .on_action(cx.listener(Explorer::delete_selected))
        .on_action(cx.listener(Explorer::copy_path))
        .on_action(cx.listener(Explorer::copy_relative_path))
        .on_action(cx.listener(Explorer::reveal_in_file_manager))
        .on_action(cx.listener(Explorer::cancel_edit_on_escape))
}

impl Explorer {
    fn target_path(&self) -> PathBuf {
        self.selected_entry()
            .map_or_else(|| self.root.clone(), |entry| entry.path.clone())
    }

    fn target_folder(&self) -> PathBuf {
        match self.selected_entry() {
            Some(entry) if entry.is_dir => entry.path.clone(),
            Some(entry) => entry
                .path
                .parent()
                .map_or_else(|| self.root.clone(), |parent| parent.to_path_buf()),
            None => self.root.clone(),
        }
    }

    fn finish_file_operation(
        &mut self,
        folder: PathBuf,
        reveal: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        self.load_directory_and_reveal(folder, reveal, cx);
        self.project
            .update(cx, |project, cx| project.files_touched(cx));
    }
}
