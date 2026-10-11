use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui_kit::{App, Context, Window};

use super::Explorer;
use super::settings::GitIgnored;
use crate::watcher::{DiskEntry, FsChanges};

impl Explorer {
    pub(super) fn apply_fs_changes(
        &mut self,
        changes: &FsChanges,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for folder in self.folders_to_reload(changes, cx) {
            self.load_directory(folder, window, cx);
        }
    }

    fn folders_to_reload(&self, changes: &FsChanges, cx: &App) -> HashSet<PathBuf> {
        if changes.rescan {
            return std::iter::once(&self.root)
                .chain(&self.expanded)
                .chain(self.children.keys())
                .cloned()
                .collect();
        }
        let mut folders = HashSet::new();
        for (path, entry) in &changes.structural {
            if self.is_tracked_folder(path) {
                folders.insert(path.clone());
            }
            if let Some(parent) = path.parent()
                && self.is_tracked_folder(parent)
                && self.listed_entry(parent, path) != *entry
            {
                folders.insert(parent.to_path_buf());
            }
        }
        if self.options.git_ignored != GitIgnored::Hide {
            folders.extend(self.folders_with_changed_ignored(changes, cx));
        }
        folders
    }

    fn folders_with_changed_ignored(&self, changes: &FsChanges, cx: &App) -> Vec<PathBuf> {
        let project = self.project.read(cx);
        changes
            .ignored
            .iter()
            .filter_map(|path| {
                let parent = path
                    .parent()
                    .filter(|parent| self.children.contains_key(*parent))?;
                let is_listed = self.listed_entry(parent, path) != DiskEntry::Missing;
                let exists = path.symlink_metadata().is_ok();
                (is_listed != exists && !project.is_excluded(path)).then(|| parent.to_path_buf())
            })
            .collect()
    }

    fn listed_entry(&self, folder: &Path, path: &Path) -> DiskEntry {
        self.children
            .get(folder)
            .and_then(|children| children.iter().find(|child| child.path == path))
            .map_or(DiskEntry::Missing, |child| DiskEntry::listed(child.is_dir))
    }

    fn is_tracked_folder(&self, path: &Path) -> bool {
        path == self.root || self.expanded.contains(path) || self.children.contains_key(path)
    }
}
