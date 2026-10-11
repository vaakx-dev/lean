use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};

use notify::RecursiveMode;

const EXCLUDE_FILE: &str = "info/exclude";

#[derive(Clone)]
pub(crate) struct MetadataWatch {
    pub(crate) dirs: Vec<PathBuf>,
    pub(crate) targets: Vec<(PathBuf, RecursiveMode)>,
    pub(crate) is_relevant: fn(&Path) -> bool,
}

impl Default for MetadataWatch {
    fn default() -> Self {
        Self {
            dirs: Vec::new(),
            targets: Vec::new(),
            is_relevant: never_relevant,
        }
    }
}

pub(super) enum MetadataPath {
    Worktree,
    Relevant,
    IgnoreRules,
    Internal,
}

#[derive(Clone, Default)]
pub(super) struct SharedMetadata(Arc<RwLock<MetadataWatch>>);

impl SharedMetadata {
    pub(super) fn replace(&self, watch: MetadataWatch) -> bool {
        let mut current = self.0.write().unwrap_or_else(PoisonError::into_inner);
        if current.dirs == watch.dirs && current.targets == watch.targets {
            return false;
        }
        *current = watch;
        true
    }

    pub(super) fn targets(
        &self,
        root: &Path,
        include_inside_root: bool,
    ) -> Vec<(PathBuf, RecursiveMode)> {
        self.read()
            .targets
            .iter()
            .filter(|(path, _)| include_inside_root || !path.starts_with(root))
            .cloned()
            .collect()
    }

    pub(super) fn classify(&self, path: &Path) -> MetadataPath {
        let watch = self.read();
        if let Some(relative) = watch
            .dirs
            .iter()
            .find_map(|dir| path.strip_prefix(dir).ok())
        {
            return if relative == Path::new(EXCLUDE_FILE) {
                MetadataPath::IgnoreRules
            } else if (watch.is_relevant)(relative) {
                MetadataPath::Relevant
            } else {
                MetadataPath::Internal
            };
        }
        match inside_dot_git(path) {
            Some(relative) if relative == Path::new(EXCLUDE_FILE) => MetadataPath::IgnoreRules,
            Some(_) => MetadataPath::Internal,
            None => MetadataPath::Worktree,
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, MetadataWatch> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }
}

fn never_relevant(_: &Path) -> bool {
    false
}

pub(super) fn is_dot_git(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new(".git"))
}

fn inside_dot_git(path: &Path) -> Option<&Path> {
    let dot_git = path.ancestors().find(|ancestor| is_dot_git(ancestor))?;
    path.strip_prefix(dot_git).ok()
}
