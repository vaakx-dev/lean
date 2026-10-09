use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub(super) struct SkippedFolders {
    root: Arc<Path>,
    folders: Arc<RwLock<HashSet<PathBuf>>>,
}

impl SkippedFolders {
    pub(super) fn new(root: &Path) -> Self {
        Self {
            root: root.into(),
            folders: Arc::default(),
        }
    }

    pub(super) fn contains_parent_of(&self, path: &Path) -> bool {
        path.parent()
            .is_some_and(|parent| self.contains_self_or_parent_of(parent))
    }

    pub(super) fn contains_self_or_parent_of(&self, path: &Path) -> bool {
        let folders = self.read();
        !folders.is_empty()
            && path
                .ancestors()
                .take_while(|ancestor| *ancestor != &*self.root && ancestor.starts_with(&self.root))
                .any(|ancestor| folders.contains(ancestor))
    }

    pub(super) fn insert(&self, folder: PathBuf) {
        self.write().insert(folder);
    }

    pub(super) fn extend(&self, folders: Vec<PathBuf>) {
        self.write().extend(folders);
    }

    pub(super) fn replace(&self, folders: HashSet<PathBuf>) {
        *self.write() = folders;
    }

    pub(super) fn snapshot(&self) -> HashSet<PathBuf> {
        self.read().clone()
    }

    fn read(&self) -> RwLockReadGuard<'_, HashSet<PathBuf>> {
        self.folders.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, HashSet<PathBuf>> {
        self.folders.write().unwrap_or_else(PoisonError::into_inner)
    }
}
