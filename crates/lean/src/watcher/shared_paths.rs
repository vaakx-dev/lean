use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Default)]
pub(super) struct SharedPaths(Arc<RwLock<HashSet<PathBuf>>>);

impl SharedPaths {
    pub(super) fn new(paths: HashSet<PathBuf>) -> Self {
        Self(Arc::new(RwLock::new(paths)))
    }

    pub(super) fn contains(&self, path: &Path) -> bool {
        self.read().contains(path)
    }

    pub(super) fn insert(&self, path: PathBuf) {
        self.write().insert(path);
    }

    pub(super) fn replace(&self, paths: HashSet<PathBuf>) -> HashSet<PathBuf> {
        std::mem::replace(&mut *self.write(), paths)
    }

    pub(super) fn read(&self) -> RwLockReadGuard<'_, HashSet<PathBuf>> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, HashSet<PathBuf>> {
        self.0.write().unwrap_or_else(PoisonError::into_inner)
    }
}
