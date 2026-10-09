use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::FsChanges;
use super::disk_entry::DiskEntry;
use super::filter::RawEvent;
use super::skipped::SkippedFolders;
use crate::project::IgnoreCache;

pub(super) fn classify(
    events: Vec<RawEvent>,
    root: &Path,
    ignore: &IgnoreCache,
    skipped: &SkippedFolders,
) -> Option<FsChanges> {
    let mut changes = FsChanges::default();
    let mut structural = HashSet::new();
    for event in events {
        changes.rescan |= event.rescan;
        changes.metadata |= event.metadata;
        changes.renames.extend(event.rename);
        if event.structural {
            structural.extend(event.paths.iter().cloned());
        }
        changes.paths.extend(event.paths);
        changes.observed_at = changes.observed_at.max(Some(event.received));
    }
    if changes.paths.iter().any(|path| is_gitignore(path)) {
        changes.rescan = true;
    }
    if changes.rescan {
        ignore.clear();
    }
    let mut folders = IgnoredFolders::new(ignore);
    let (ignored, paths): (HashSet<PathBuf>, HashSet<PathBuf>) = std::mem::take(&mut changes.paths)
        .into_iter()
        .partition(|path| folders.contains_path(path));
    for path in &ignored {
        if let Some(folder) = folders.topmost_parent(path, root) {
            skipped.insert(folder);
        }
    }
    changes.structural = paths
        .iter()
        .filter(|path| structural.contains(*path))
        .map(|path| (path.clone(), DiskEntry::of(path)))
        .collect();
    changes.paths = paths;
    changes.ignored = ignored;
    (!changes.is_empty()).then_some(changes)
}

fn is_gitignore(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new(".gitignore"))
}

struct IgnoredFolders<'a> {
    ignore: &'a IgnoreCache,
    known: HashMap<PathBuf, bool>,
}

impl<'a> IgnoredFolders<'a> {
    fn new(ignore: &'a IgnoreCache) -> Self {
        Self {
            ignore,
            known: HashMap::new(),
        }
    }

    fn contains_path(&mut self, path: &Path) -> bool {
        path.parent().is_some_and(|folder| self.contains(folder)) || self.ignore.is_ignored(path)
    }

    fn topmost_parent(&mut self, path: &Path, root: &Path) -> Option<PathBuf> {
        path.ancestors()
            .skip(1)
            .take_while(|folder| *folder != root && folder.starts_with(root))
            .filter(|folder| self.contains(folder))
            .last()
            .map(Path::to_path_buf)
    }

    fn contains(&mut self, folder: &Path) -> bool {
        if let Some(ignored) = self.known.get(folder) {
            return *ignored;
        }
        let ignored = self.ignore.is_ignored(folder);
        self.known.insert(folder.to_path_buf(), ignored);
        ignored
    }
}
