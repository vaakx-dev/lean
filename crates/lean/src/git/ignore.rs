use std::path::{Path, PathBuf};

pub(crate) struct IgnoreRules {
    repository: git2::Repository,
    workdir: PathBuf,
}

impl IgnoreRules {
    pub(crate) fn discover(path: &Path) -> Option<Self> {
        let repository = git2::Repository::discover(path).ok()?;
        let workdir = repository.workdir()?;
        let workdir = dunce::canonicalize(workdir).unwrap_or_else(|error| {
            log::debug!("failed to canonicalize {}: {error}", workdir.display());
            workdir.to_path_buf()
        });
        Some(Self {
            repository,
            workdir,
        })
    }

    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        path.strip_prefix(&self.workdir).is_ok_and(|relative_path| {
            self.repository
                .is_path_ignored(relative_path)
                .unwrap_or_else(|error| {
                    log::debug!(
                        "failed to check ignore rules for {}: {error}",
                        path.display()
                    );
                    false
                })
        })
    }
}
