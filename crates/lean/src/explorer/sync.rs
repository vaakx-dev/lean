use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gpui_kit::{Context, Window};

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
        if changes.rescan {
            if !self.has_read_since(changes) {
                self.reload_all(window, cx);
            }
            return;
        }
        let folders = self.folders_with_changed_entries(changes);
        let listed = self.listed_ignored_paths(changes);
        if listed.is_empty() {
            self.load_directories(folders.into_iter().collect(), window, cx);
            return;
        }
        let presence = self.project.read(cx).changed_presence(listed, cx);
        cx.spawn_in(window, async move |explorer, cx| {
            let mut folders = folders;
            let changed = presence.await;
            folders.extend(
                changed
                    .iter()
                    .filter_map(|path| path.parent().map(Path::to_path_buf)),
            );
            explorer
                .update_in(cx, |explorer, window, cx| {
                    explorer.load_directories(folders.into_iter().collect(), window, cx)
                })
                .ok();
        })
        .detach();
    }

    fn has_read_since(&self, changes: &FsChanges) -> bool {
        changes
            .observed_at
            .is_some_and(|observed| self.opened_at > observed)
    }

    fn reload_all(&mut self, window: &Window, cx: &mut Context<Self>) {
        let root = self.root.clone();
        let expanded = &self.expanded;
        self.children
            .retain(|folder, _| *folder == root || expanded.contains(folder));
        let folders = std::iter::once(root)
            .chain(self.expanded.iter().cloned())
            .collect();
        self.load_directories(folders, window, cx);
    }

    fn folders_with_changed_entries(&self, changes: &FsChanges) -> HashSet<PathBuf> {
        let mut folders = HashSet::new();
        let mut by_parent: HashMap<&Path, Vec<(&Path, DiskEntry)>> = HashMap::new();
        for (path, entry) in &changes.structural {
            if self.is_tracked_folder(path) {
                folders.insert(path.clone());
            }
            if let Some(parent) = path
                .parent()
                .filter(|parent| self.is_tracked_folder(parent))
            {
                by_parent
                    .entry(parent)
                    .or_default()
                    .push((path.as_path(), *entry));
            }
        }
        for (parent, changed) in by_parent {
            let listing = self.listing(parent);
            let differs = changed
                .iter()
                .any(|(path, entry)| listed_entry(&listing, path) != *entry);
            if differs {
                folders.insert(parent.to_path_buf());
            }
        }
        folders
    }

    fn listed_ignored_paths(&self, changes: &FsChanges) -> Vec<(PathBuf, bool)> {
        if self.options.git_ignored == GitIgnored::Hide {
            return Vec::new();
        }
        let mut listings: HashMap<&Path, HashMap<&Path, DiskEntry>> = HashMap::new();
        changes
            .ignored
            .iter()
            .filter_map(|path| {
                let parent = path
                    .parent()
                    .filter(|parent| self.children.contains_key(*parent))?;
                let listing = listings
                    .entry(parent)
                    .or_insert_with(|| self.listing(parent));
                let is_listed = listed_entry(listing, path) != DiskEntry::Missing;
                Some((path.clone(), is_listed))
            })
            .collect()
    }

    fn listing(&self, folder: &Path) -> HashMap<&Path, DiskEntry> {
        self.children
            .get(folder)
            .into_iter()
            .flatten()
            .map(|child| (&*child.path, DiskEntry::listed(child.is_dir)))
            .collect()
    }

    fn is_tracked_folder(&self, path: &Path) -> bool {
        path == self.root || self.expanded.contains(path) || self.children.contains_key(path)
    }
}

fn listed_entry(listing: &HashMap<&Path, DiskEntry>, path: &Path) -> DiskEntry {
    listing.get(path).copied().unwrap_or(DiskEntry::Missing)
}
