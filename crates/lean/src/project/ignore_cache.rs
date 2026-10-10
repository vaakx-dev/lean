use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::exclusions::Exclusions;
use super::ignore_rules::IgnoreRules;

type SharedRules = Arc<Mutex<IgnoreRules>>;

#[derive(Clone)]
pub(crate) struct IgnoreCache {
    root: PathBuf,
    exclusions: Exclusions,
    repositories: Arc<Mutex<HashMap<PathBuf, SharedRules>>>,
}

impl IgnoreCache {
    pub(super) fn new(root: PathBuf, excluded: &[String]) -> Self {
        Self {
            exclusions: Exclusions::new(root.clone(), excluded),
            root,
            repositories: Arc::default(),
        }
    }

    pub(super) fn set_exclusions(&self, patterns: &[String]) {
        self.exclusions.set_patterns(patterns);
    }

    pub(crate) fn clear(&self) {
        self.repositories().clear();
    }

    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        if path == self.root || !path.starts_with(&self.root) {
            return false;
        }
        let is_dir = path.is_dir();
        if self.exclusions.contains(path, is_dir) {
            return true;
        }
        let Some(folder) = path.parent() else {
            return false;
        };
        self.with_rules(folder, |rules| rules.is_ignored(path, is_dir))
    }

    pub(super) fn is_excluded(&self, path: &Path, is_dir: bool) -> bool {
        self.exclusions.contains(path, is_dir)
    }

    pub(crate) fn split_ignored_folders(
        &self,
        folder: &Path,
        folders: Vec<PathBuf>,
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        self.with_rules(folder, |rules| {
            folders.into_iter().partition(|path| {
                self.exclusions.contains(path, true) || rules.is_ignored(path, true)
            })
        })
    }

    pub(super) fn with_rules<R>(
        &self,
        folder: &Path,
        read: impl FnOnce(&mut IgnoreRules) -> R,
    ) -> R {
        let rules = self.rules_for(self.repository_root(folder));
        let mut rules = rules.lock().unwrap_or_else(PoisonError::into_inner);
        read(&mut rules)
    }

    fn rules_for(&self, repository_root: &Path) -> SharedRules {
        if let Some(rules) = self.repositories().get(repository_root) {
            return rules.clone();
        }
        let discovered = Arc::new(Mutex::new(IgnoreRules::discover(repository_root)));
        self.repositories()
            .entry(repository_root.to_path_buf())
            .or_insert(discovered)
            .clone()
    }

    fn repositories(&self) -> MutexGuard<'_, HashMap<PathBuf, SharedRules>> {
        self.repositories
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn repository_root<'a>(&'a self, folder: &'a Path) -> &'a Path {
        folder
            .ancestors()
            .take_while(|ancestor| ancestor.starts_with(&self.root))
            .find(|ancestor| ancestor.join(".git").exists())
            .unwrap_or(self.root.as_path())
    }
}
