use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::ignore_rules::IgnoreRules;

type SharedRules = Arc<Option<Mutex<IgnoreRules>>>;

#[derive(Clone)]
pub(crate) struct IgnoreCache {
    root: PathBuf,
    repositories: Arc<Mutex<HashMap<PathBuf, SharedRules>>>,
}

impl IgnoreCache {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            root,
            repositories: Arc::default(),
        }
    }

    pub(crate) fn clear(&self) {
        self.repositories().clear();
    }

    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        if path == self.root || !path.starts_with(&self.root) {
            return false;
        }
        let Some(folder) = path.parent() else {
            return false;
        };
        self.with_rules(folder, |rules| {
            rules.is_some_and(|rules| rules.is_ignored(path, path.is_dir()))
        })
    }

    pub(crate) fn split_ignored_folders(
        &self,
        folder: &Path,
        folders: Vec<PathBuf>,
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        self.with_rules(folder, |rules| match rules {
            Some(rules) => folders
                .into_iter()
                .partition(|path| rules.is_ignored(path, true)),
            None => (Vec::new(), folders),
        })
    }

    pub(super) fn with_rules<R>(
        &self,
        folder: &Path,
        read: impl FnOnce(Option<&mut IgnoreRules>) -> R,
    ) -> R {
        let rules = self.rules_for(self.repository_root(folder));
        match rules.as_ref() {
            Some(rules) => {
                let mut rules = rules.lock().unwrap_or_else(PoisonError::into_inner);
                read(Some(&mut *rules))
            }
            None => read(None),
        }
    }

    fn rules_for(&self, repository_root: &Path) -> SharedRules {
        if let Some(rules) = self.repositories().get(repository_root) {
            return rules.clone();
        }
        let discovered = Arc::new(IgnoreRules::discover(repository_root).map(Mutex::new));
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
