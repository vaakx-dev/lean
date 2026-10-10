use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::settings::Setting;

pub(crate) const EXCLUDE: Setting<Vec<String>> =
    Setting::new_with("files.exclude", default_exclude);

fn default_exclude() -> Vec<String> {
    ["node_modules", "target", ".venv", "__pycache__"]
        .map(String::from)
        .to_vec()
}

#[derive(Clone)]
pub(super) struct Exclusions {
    root: PathBuf,
    matcher: Arc<RwLock<Gitignore>>,
}

impl Exclusions {
    pub(super) fn new(root: PathBuf, patterns: &[String]) -> Self {
        let matcher = build_matcher(&root, patterns);
        Self {
            root,
            matcher: Arc::new(RwLock::new(matcher)),
        }
    }

    pub(super) fn set_patterns(&self, patterns: &[String]) {
        let matcher = build_matcher(&self.root, patterns);
        *self.matcher.write().unwrap_or_else(PoisonError::into_inner) = matcher;
    }

    pub(super) fn contains(&self, path: &Path, is_dir: bool) -> bool {
        if path == self.root || !path.starts_with(&self.root) {
            return false;
        }
        self.matcher
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .matched_path_or_any_parents(path, is_dir)
            .is_ignore()
    }
}

fn build_matcher(root: &Path, patterns: &[String]) -> Gitignore {
    let mut builder = GitignoreBuilder::new(root);
    for pattern in patterns {
        if let Err(error) = builder.add_line(None, pattern) {
            log::warn!("invalid excluded path pattern {pattern:?}: {error}");
        }
    }
    builder.build().unwrap_or_else(|error| {
        log::warn!("failed to build excluded path patterns: {error}");
        Gitignore::empty()
    })
}
