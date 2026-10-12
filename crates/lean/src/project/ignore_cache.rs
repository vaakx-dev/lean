use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::exclusions::Exclusions;
use super::ignore_rules::{IgnoreRules, is_repository};

type SharedRules = Arc<Mutex<IgnoreRules>>;

#[derive(Default)]
struct Cache {
    workdirs: HashMap<PathBuf, Arc<Path>>,
    rules: HashMap<Arc<Path>, SharedRules>,
}

#[derive(Clone)]
pub(crate) struct IgnoreCache {
    root: Arc<Path>,
    exclusions: Exclusions,
    cache: Arc<Mutex<Cache>>,
}

impl IgnoreCache {
    pub(super) fn new(root: PathBuf, excluded: &[String]) -> Self {
        Self {
            exclusions: Exclusions::new(root.clone(), excluded),
            root: root.into(),
            cache: Arc::default(),
        }
    }

    pub(super) fn set_exclusions(&self, patterns: &[String]) {
        self.exclusions.set_patterns(patterns);
    }

    pub(crate) fn clear(&self) {
        *self.cache() = Cache::default();
    }

    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        let Some(folder) = self.parent_inside_root(path) else {
            return false;
        };
        self.with_rules(folder, |rules| {
            let mut matches =
                |is_dir| self.exclusions.contains(path, is_dir) || rules.is_ignored(path, is_dir);
            let as_file = matches(false);
            let as_folder = matches(true);
            if as_file == as_folder || !path.is_dir() {
                as_file
            } else {
                as_folder
            }
        })
    }

    pub(crate) fn is_ignored_folder(&self, folder: &Path) -> bool {
        let Some(parent) = self.parent_inside_root(folder) else {
            return false;
        };
        self.with_rules(parent, |rules| {
            self.exclusions.contains(folder, true) || rules.is_ignored(folder, true)
        })
    }

    pub(crate) fn is_inside_excluded(&self, path: &Path) -> bool {
        path.parent()
            .is_some_and(|parent| self.exclusions.contains(parent, true))
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
        let rules = self.rules_for(folder);
        let mut rules = rules.lock().unwrap_or_else(PoisonError::into_inner);
        read(&mut rules)
    }

    fn parent_inside_root<'a>(&self, path: &'a Path) -> Option<&'a Path> {
        if path == &*self.root || !path.starts_with(&self.root) {
            return None;
        }
        path.parent()
    }

    fn rules_for(&self, folder: &Path) -> SharedRules {
        let workdir = self.workdir(folder);
        if let Some(rules) = self.cache().rules.get(&workdir) {
            return rules.clone();
        }
        let created = Arc::new(Mutex::new(IgnoreRules::new(&workdir)));
        self.cache().rules.entry(workdir).or_insert(created).clone()
    }

    fn workdir(&self, folder: &Path) -> Arc<Path> {
        if !folder.starts_with(&self.root) {
            return self.workdir(&self.root);
        }
        if let Some(workdir) = self.cache().workdirs.get(folder) {
            return workdir.clone();
        }
        let parent = folder.parent().filter(|_| folder != &*self.root);
        let workdir = if is_repository(folder) {
            Arc::from(folder)
        } else if let Some(parent) = parent {
            self.workdir(parent)
        } else {
            self.enclosing_repository()
        };
        self.cache()
            .workdirs
            .insert(folder.to_path_buf(), workdir.clone());
        workdir
    }

    fn enclosing_repository(&self) -> Arc<Path> {
        self.root
            .ancestors()
            .skip(1)
            .find(|ancestor| is_repository(ancestor))
            .map_or_else(|| self.root.clone(), |repository| Arc::from(repository))
    }

    fn cache(&self) -> MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
