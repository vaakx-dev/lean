mod create;
mod delete;
mod edit_confirmation;
mod edit_field;
mod inline_edit;
mod name_validation;
mod rename;

use std::fmt::Display;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    App, AppContext as _, ClipboardItem, Context, Div, InteractiveElement as _, PromptLevel,
    Stateful, Window, actions,
};

use super::Explorer;
use crate::files::file_name;
use crate::project::ProjectEvent;

pub(super) use inline_edit::InlineEdit;

const CONFIRM_ANSWER: usize = 0;

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
    fn copy_path(&mut self, _: &CopyPath, _: &mut Window, cx: &mut Context<Self>) {
        let path = self.target_path();
        cx.write_to_clipboard(ClipboardItem::new_string(
            path.to_string_lossy().into_owned(),
        ));
    }

    fn copy_relative_path(&mut self, _: &CopyRelativePath, _: &mut Window, cx: &mut Context<Self>) {
        let path = self.target_path();
        let Ok(relative) = path.strip_prefix(&self.root) else {
            return;
        };
        if relative.as_os_str().is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(
            relative.to_string_lossy().into_owned(),
        ));
    }

    fn reveal_in_file_manager(
        &mut self,
        _: &RevealInFileManager,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.reveal_path(&self.target_path());
    }

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

    fn run_disk_op<E: Send + 'static>(
        &mut self,
        work: impl FnOnce() -> Result<(), E> + Send + 'static,
        window: &Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, Result<(), E>, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let task = cx.background_spawn(async move { work() });
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            explorer
                .update_in(cx, |explorer, window, cx| {
                    then(explorer, result, window, cx)
                })
                .ok();
        })
        .detach();
    }

    fn confirm(
        &mut self,
        message: &str,
        detail: &str,
        button: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let answer = window.prompt(
            PromptLevel::Warning,
            message,
            Some(detail),
            &[button, "Cancel"],
            cx,
        );
        cx.spawn_in(window, async move |explorer, cx| {
            if !matches!(answer.await, Ok(CONFIRM_ANSWER)) {
                return;
            }
            explorer
                .update_in(cx, |explorer, window, cx| then(explorer, window, cx))
                .ok();
        })
        .detach();
    }

    fn finish_file_operation(
        &mut self,
        folder: PathBuf,
        reveal: Option<PathBuf>,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        self.load_directory_and_reveal(folder, reveal, window, cx);
        self.project
            .update(cx, |_, cx| cx.emit(ProjectEvent::FilesTouched));
    }
}

fn report_failure(verb: &str, path: &Path, error: impl Display, window: &mut Window, cx: &mut App) {
    log::warn!("failed to {verb} {}: {error}", path.display());
    window.push_notification(format!("Can't {verb} \"{}\": {error}", file_name(path)), cx);
}
