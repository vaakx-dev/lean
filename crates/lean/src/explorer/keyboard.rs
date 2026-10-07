use gpui_kit::{App, Context, KeyBinding, ScrollStrategy, Window, actions};

use super::{Explorer, ExplorerEvent};

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

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, Some(KEY_CONTEXT)),
        KeyBinding::new("up", SelectPrevious, Some(KEY_CONTEXT)),
        KeyBinding::new("right", ExpandSelected, Some(KEY_CONTEXT)),
        KeyBinding::new("left", CollapseSelected, Some(KEY_CONTEXT)),
        KeyBinding::new("enter", OpenSelected, Some(KEY_CONTEXT)),
        KeyBinding::new("space", OpenSelected, Some(KEY_CONTEXT)),
    ]);
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
        _: &mut Window,
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
            self.expand_folder(path, cx);
        }
    }

    pub(super) fn collapse_selected(
        &mut self,
        _: &CollapseSelected,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self
            .selected_index()
            .and_then(|index| self.visible.get(index))
        else {
            return;
        };
        let path = entry.path.clone();
        if entry.is_dir && self.expanded.contains(&path) {
            self.toggle_folder(path, cx);
            return;
        }
        let parent_index = path
            .parent()
            .and_then(|parent| self.visible.iter().position(|entry| entry.path == parent));
        if let Some(parent_index) = parent_index {
            self.select_index(parent_index, cx);
        }
    }

    pub(super) fn open_selected(
        &mut self,
        _: &OpenSelected,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(entry) = self
            .selected_index()
            .and_then(|index| self.visible.get(index))
        else {
            return;
        };
        let path = entry.path.clone();
        if entry.is_dir {
            self.toggle_folder(path, cx);
        } else {
            cx.emit(ExplorerEvent::OpenFile(path));
        }
    }

    fn selected_index(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        self.visible
            .iter()
            .position(|entry| &entry.path == selected)
    }

    fn select_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(entry) = self.visible.get(index) {
            self.selected = Some(entry.path.clone());
            self.scroll_handle
                .scroll_to_item(index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }
}
