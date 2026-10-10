use std::ops::Range;
use std::path::{Path, PathBuf};

use gpui_kit::component::input::{Escape, InputState};
use gpui_kit::{
    AppContext as _, Context, Entity, Focusable as _, ScrollStrategy, SharedString, Subscription,
    Window,
};

use super::{NewFile, NewFolder, RenameSelected};
use crate::explorer::{Explorer, VisibleEntry};
use crate::files::file_name;

pub(super) enum EditKind {
    NewFile,
    NewFolder,
    Rename {
        path: PathBuf,
        original_name: SharedString,
    },
}

pub(in crate::explorer) struct InlineEdit {
    pub(super) kind: EditKind,
    pub(super) folder: PathBuf,
    pub(super) input: Entity<InputState>,
    pub(super) error: Option<SharedString>,
    _subscription: Subscription,
}

impl Explorer {
    pub(in crate::explorer) fn new_file(
        &mut self,
        _: &NewFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_create(EditKind::NewFile, window, cx);
    }

    pub(in crate::explorer) fn new_folder(
        &mut self,
        _: &NewFolder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_create(EditKind::NewFolder, window, cx);
    }

    pub(super) fn rename_selected(
        &mut self,
        _: &RenameSelected,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let Some(folder) = entry.path.parent().map(Path::to_path_buf) else {
            return;
        };
        let name = file_name(&entry.path);
        let selection = 0..stem_length(&name, entry.is_dir);
        let kind = EditKind::Rename {
            path: entry.path.clone(),
            original_name: name.clone(),
        };
        self.begin_edit(kind, folder, name, selection, window, cx);
    }

    pub(super) fn cancel_edit_on_escape(
        &mut self,
        _: &Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.editing.is_none() {
            cx.propagate();
            return;
        }
        self.cancel_edit(window, cx);
    }

    pub(in crate::explorer) fn cancel_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit) = self.editing.take() {
            self.close_edit(&edit, window, cx);
        }
    }

    pub(in crate::explorer) fn edit_for(&self, entry: &VisibleEntry) -> Option<&InlineEdit> {
        let edit = self.editing.as_ref()?;
        let is_edited = match &edit.kind {
            EditKind::Rename { path, .. } => !entry.is_draft && &entry.path == path,
            EditKind::NewFile | EditKind::NewFolder => entry.is_draft,
        };
        is_edited.then_some(edit)
    }

    pub(in crate::explorer) fn draft_in(&self, folder: &Path) -> Option<bool> {
        let edit = self.editing.as_ref().filter(|edit| edit.folder == folder)?;
        match edit.kind {
            EditKind::NewFile => Some(false),
            EditKind::NewFolder => Some(true),
            EditKind::Rename { .. } => None,
        }
    }

    fn start_create(&mut self, kind: EditKind, window: &mut Window, cx: &mut Context<Self>) {
        let folder = self.target_folder();
        if folder == self.root {
            self.root_expanded = true;
        } else {
            self.expand_folder(folder.clone(), cx);
        }
        self.begin_edit(kind, folder, SharedString::default(), 0..0, window, cx);
    }

    fn begin_edit(
        &mut self,
        kind: EditKind,
        folder: PathBuf,
        name: SharedString,
        selection: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(previous) = self.editing.take() {
            self.close_edit(&previous, window, cx);
        }
        let input = cx.new(|cx| InputState::new(window, cx).context_menu(false));
        input.update(cx, |input, cx| {
            input.set_value(name, window, cx);
            input.set_selected_range(selection, cx);
        });
        let subscription = cx.subscribe_in(&input, window, Self::handle_edit_event);
        self.editing = Some(InlineEdit {
            kind,
            folder,
            input,
            error: None,
            _subscription: subscription,
        });
        self.rebuild_visible();
        self.scroll_to_edit();
        cx.defer_in(window, |explorer, window, cx| {
            explorer.focus_edit(window, cx)
        });
        cx.notify();
    }

    pub(super) fn close_edit(
        &mut self,
        edit: &InlineEdit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if edit.input.focus_handle(cx).contains_focused(window, cx) {
            window.focus(&self.focus_handle, cx);
        }
        self.rebuild_visible();
        cx.notify();
    }

    fn focus_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(input) = self.editing.as_ref().map(|edit| edit.input.clone()) {
            input.update(cx, |input, cx| input.focus(window, cx));
        }
    }

    fn scroll_to_edit(&self) {
        let index = self
            .visible
            .iter()
            .position(|entry| self.edit_for(entry).is_some());
        if let Some(index) = index {
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
        }
    }
}

fn stem_length(name: &str, is_dir: bool) -> usize {
    match name.rfind('.') {
        Some(index) if index > 0 && !is_dir => index,
        _ => name.len(),
    }
}
