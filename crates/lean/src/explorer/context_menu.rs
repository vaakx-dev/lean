use gpui_kit::component::menu::PopupMenu;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{Context, MouseButton, MouseDownEvent, Window};

use super::Explorer;
use super::file_operations::{
    CopyPath, CopyRelativePath, DeleteSelected, NewFile, NewFolder, RenameSelected,
    RevealInFileManager,
};
use super::keyboard::OpenSelected;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuTarget {
    Root,
    Folder,
    File,
}

impl Explorer {
    pub(super) fn prepare_context_menu(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Right {
            return;
        }
        self.cancel_edit(window, cx);
        window.focus(&self.focus_handle, cx);
    }

    pub(super) fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected = None;
        cx.notify();
    }

    pub(super) fn menu_target(&self) -> MenuTarget {
        match self.selected_entry() {
            Some(entry) if entry.is_dir => MenuTarget::Folder,
            Some(_) => MenuTarget::File,
            None => MenuTarget::Root,
        }
    }
}

pub(super) fn build_context_menu(menu: PopupMenu, target: MenuTarget) -> PopupMenu {
    let is_root = target == MenuTarget::Root;
    menu.when(target == MenuTarget::File, |menu| {
        menu.menu("Open", Box::new(OpenSelected)).separator()
    })
    .menu("New File…", Box::new(NewFile))
    .menu("New Folder…", Box::new(NewFolder))
    .separator()
    .menu("Reveal in File Manager", Box::new(RevealInFileManager))
    .separator()
    .menu("Copy Path", Box::new(CopyPath))
    .when(!is_root, |menu| {
        menu.menu("Copy Relative Path", Box::new(CopyRelativePath))
            .separator()
            .menu("Rename…", Box::new(RenameSelected))
            .menu("Delete", Box::new(DeleteSelected))
    })
}
