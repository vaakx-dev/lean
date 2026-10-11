use std::path::Path;

use gpui_kit::{Context, Window};

use super::{Explorer, VisibleEntry};
use crate::project::DirectoryEntry;

const MAX_UNFOLD_DEPTH: usize = 32;

impl Explorer {
    pub(super) fn entry_row(&self, entry: &DirectoryEntry, depth: usize) -> VisibleEntry {
        let mut row = VisibleEntry {
            path: entry.path.clone(),
            name: entry.name.clone(),
            depth,
            is_dir: entry.is_dir,
            is_draft: false,
            is_ignored: entry.is_ignored,
        };
        if !entry.is_dir || !self.options.compact_folders {
            return row;
        }
        let mut name = entry.name.to_string();
        while let Some(next) = self.only_subfolder(&row.path) {
            name.push('/');
            name.push_str(&next.name);
            row.path = next.path.clone();
            row.is_ignored = next.is_ignored;
        }
        row.name = name.into();
        row
    }

    pub(super) fn unfold_single_child(
        &mut self,
        folder: &Path,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let nesting = folder
            .components()
            .count()
            .saturating_sub(self.root.components().count());
        if !self.options.compact_folders || nesting > MAX_UNFOLD_DEPTH {
            return;
        }
        let Some(child) = self.only_subfolder(folder).map(|entry| entry.path.clone()) else {
            return;
        };
        if !self.expanded.contains(&child) {
            self.expand_folder(child, window, cx);
        }
    }

    fn only_subfolder(&self, folder: &Path) -> Option<&DirectoryEntry> {
        if !self.expanded.contains(folder) || self.draft_in(folder).is_some() {
            return None;
        }
        let mut shown = self.shown_children(folder);
        let only = shown.next().filter(|entry| entry.is_dir)?;
        shown.next().is_none().then_some(only)
    }
}
