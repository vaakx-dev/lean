use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::shared_paths::SharedPaths;

#[derive(Clone)]
pub(super) struct SkippedFolders {
    root: Arc<Path>,
    folders: SharedPaths,
}

impl SkippedFolders {
    pub(super) fn new(root: &Path) -> Self {
        Self {
            root: root.into(),
            folders: SharedPaths::default(),
        }
    }

    pub(super) fn contains_parent_of(&self, path: &Path) -> bool {
        path.parent()
            .is_some_and(|parent| self.contains_self_or_parent_of(parent))
    }

    pub(super) fn contains_self_or_parent_of(&self, path: &Path) -> bool {
        let folders = self.folders.read();
        !folders.is_empty()
            && path
                .ancestors()
                .take_while(|ancestor| *ancestor != &*self.root && ancestor.starts_with(&self.root))
                .any(|ancestor| folders.contains(ancestor))
    }

    pub(super) fn insert(&self, folder: PathBuf) {
        self.folders.insert(folder);
    }

    pub(super) fn extend(&self, folders: Vec<PathBuf>) {
        self.folders.extend(folders);
    }

    pub(super) fn replace(&self, folders: HashSet<PathBuf>) {
        self.folders.replace(folders);
    }

    pub(super) fn snapshot(&self) -> HashSet<PathBuf> {
        self.folders.read().clone()
    }
}
