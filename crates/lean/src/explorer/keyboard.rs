use gpui_kit::{App, Context, KeyBinding, ScrollStrategy, Window, actions};

use super::file_operations::{DeleteSelected, RenameSelected};
use super::{Explorer, VisibleEntry};
use crate::app::keymap;
use crate::plugins::PanelEvent;

actions!(
    explorer,
    [
        SelectNext,
        SelectPrevious,
        ExpandSelected,
        CollapseSelected,
        OpenSelected
    ]
);

pub(super) const KEY_CONTEXT: &str = "Explorer";
pub(super) const NOT_EDITING_CONTEXT: &str = "not_editing";
const NAVIGATION_CONTEXT: &str = "Explorer && not_editing";

pub(crate) fn bind_keys(cx: &mut App) {
    keymap::bind_defaults(
        cx,
        [
            KeyBinding::new("down", SelectNext, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("up", SelectPrevious, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("right", ExpandSelected, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("left", CollapseSelected, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("space", OpenSelected, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("enter", OpenSelected, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("f2", RenameSelected, Some(NAVIGATION_CONTEXT)),
            KeyBinding::new("delete", DeleteSelected, Some(NAVIGATION_CONTEXT)),
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-backspace", DeleteSelected, Some(NAVIGATION_CONTEXT)),
        ],
    );
}

impl Explorer {
    pub(super) fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        let last = self.visible.len().saturating_sub(1);
        let index = self
            .selected_index()
            .map_or(0, |index| (index + 1).min(last));
        self.select_index(index, cx);
    }

    pub(super) fn select_previous(
        &mut self,
        _: &SelectPrevious,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = self
            .selected_index()
            .map_or(0, |index| index.saturating_sub(1));
        self.select_index(index, cx);
    }

    pub(super) fn expand_selected(
        &mut self,
        _: &ExpandSelected,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self.selected_index() else {
            return;
        };
        let Some(entry) = self.visible.get(index) else {
            return;
        };
        if !entry.is_dir {
            return;
        }
        let path = entry.path.clone();
        if self.expanded.contains(&path) {
            self.select_index(index + 1, cx);
        } else {
            self.expand_folder(path, window, cx);
        }
    }

    pub(super) fn collapse_selected(
        &mut self,
        _: &CollapseSelected,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let path = entry.path.clone();
        if entry.is_dir && self.expanded.contains(&path) {
            self.toggle_folder(path, window, cx);
            return;
        }
        let parent_index = path.ancestors().skip(1).find_map(|ancestor| {
            self.visible
                .iter()
                .position(|entry| !entry.is_draft && entry.path == ancestor)
        });
        if let Some(parent_index) = parent_index {
            self.select_index(parent_index, cx);
        }
    }

    pub(super) fn open_selected(
        &mut self,
        _: &OpenSelected,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let path = entry.path.clone();
        if entry.is_dir {
            self.toggle_folder(path, window, cx);
        } else {
            cx.emit(PanelEvent::OpenFile(path));
        }
    }

    pub(super) fn selected_index(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        self.visible
            .iter()
            .position(|entry| !entry.is_draft && &entry.path == selected)
    }

    pub(super) fn selected_entry(&self) -> Option<&VisibleEntry> {
        self.selected_index()
            .and_then(|index| self.visible.get(index))
    }

    pub(super) fn select_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(entry) = self.visible.get(index) {
            self.selected = Some(entry.path.clone());
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
}
