use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder, gitconfig_excludes_path};

pub(crate) struct IgnoreRules {
    workdir: PathBuf,
    shared: Gitignore,
    folders: HashMap<PathBuf, Option<Gitignore>>,
    ignored_dirs: HashMap<PathBuf, bool>,
}

impl IgnoreRules {
    pub(crate) fn discover(path: &Path) -> Option<Self> {
        let workdir = path
            .ancestors()
            .find(|ancestor| ancestor.join(".git").exists())?;
        Some(Self {
            shared: shared_rules(workdir),
            workdir: workdir.to_path_buf(),
            folders: HashMap::new(),
            ignored_dirs: HashMap::new(),
        })
    }

    pub(crate) fn is_ignored(&mut self, path: &Path, is_dir: bool) -> bool {
        if path == self.workdir || !path.starts_with(&self.workdir) {
            return false;
        }
        let parent_ignored = path
            .parent()
            .is_some_and(|parent| self.is_ignored_dir(parent));
        parent_ignored || self.matches(path, is_dir)
    }

    fn is_ignored_dir(&mut self, dir: &Path) -> bool {
        if dir == self.workdir || !dir.starts_with(&self.workdir) {
            return false;
        }
        if let Some(ignored) = self.ignored_dirs.get(dir) {
            return *ignored;
        }
        let ignored = self.is_ignored(dir, true);
        self.ignored_dirs.insert(dir.to_path_buf(), ignored);
        ignored
    }

    fn matches(&mut self, path: &Path, is_dir: bool) -> bool {
        let mut folder = path.parent();
        while let Some(current) = folder.filter(|folder| folder.starts_with(&self.workdir)) {
            let matched = self
                .folder_rules(current)
                .map(|rules| rules.matched(path, is_dir));
            match matched {
                Some(Match::Ignore(_)) => return true,
                Some(Match::Whitelist(_)) => return false,
                Some(Match::None) | None => folder = current.parent(),
            }
        }
        self.shared.matched(path, is_dir).is_ignore()
    }

    fn folder_rules(&mut self, folder: &Path) -> Option<&Gitignore> {
        if !self.folders.contains_key(folder) {
            self.folders
                .insert(folder.to_path_buf(), folder_rules(folder));
        }
        self.folders.get(folder).and_then(Option::as_ref)
    }
}

fn folder_rules(folder: &Path) -> Option<Gitignore> {
    let path = folder.join(".gitignore");
    if !path.is_file() {
        return None;
    }
    let (rules, error) = Gitignore::new(&path);
    if let Some(error) = error {
        log::debug!("failed to read {}: {error}", path.display());
    }
    Some(rules)
}

fn shared_rules(workdir: &Path) -> Gitignore {
    let mut builder = GitignoreBuilder::new(workdir);
    let files = gitconfig_excludes_path()
        .into_iter()
        .chain(git_dir(workdir).map(|dir| dir.join("info").join("exclude")))
        .filter(|path| path.is_file());
    for path in files {
        if let Some(error) = builder.add(&path) {
            log::debug!("failed to read {}: {error}", path.display());
        }
    }
    builder.build().unwrap_or_else(|error| {
        log::debug!(
            "failed to build ignore rules for {}: {error}",
            workdir.display()
        );
        Gitignore::empty()
    })
}

fn git_dir(workdir: &Path) -> Option<PathBuf> {
    let dot_git = workdir.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let contents = std::fs::read_to_string(&dot_git).ok()?;
    let linked = workdir.join(contents.strip_prefix("gitdir:")?.trim());
    match std::fs::read_to_string(linked.join("commondir")) {
        Ok(common) => Some(linked.join(common.trim())),
        Err(_) => Some(linked),
    }
}
