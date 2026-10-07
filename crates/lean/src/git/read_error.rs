use std::path::{Path, PathBuf};

use gpui_kit::SharedString;

#[derive(Clone)]
pub(crate) struct GitReadError {
    pub(crate) message: SharedString,
    pub(crate) trust_command: Option<SharedString>,
}

impl GitReadError {
    pub(crate) fn new(error: &anyhow::Error, root: &Path) -> Self {
        let git_error = error.downcast_ref::<git2::Error>();
        let is_untrusted_owner =
            git_error.is_some_and(|error| error.code() == git2::ErrorCode::Owner);
        let message = match git_error {
            Some(git_error) => git_error.message().to_string(),
            None => format!("{error:#}"),
        };
        Self {
            message: message.into(),
            trust_command: is_untrusted_owner.then(|| trust_command(root)),
        }
    }
}

fn trust_command(root: &Path) -> SharedString {
    let folder = repository_folder(root).unwrap_or_else(|| root.to_path_buf());
    let folder = folder.to_string_lossy().replace('\\', "/");
    format!("git config --global --add safe.directory \"{folder}\"").into()
}

fn repository_folder(root: &Path) -> Option<PathBuf> {
    let git_folder = git2::Repository::discover_path(root, std::iter::empty::<&Path>()).ok()?;
    let git_folder = dunce::canonicalize(&git_folder).unwrap_or_else(|error| {
        log::debug!("failed to canonicalize {}: {error}", git_folder.display());
        git_folder.clone()
    });
    git_folder.parent().map(Path::to_path_buf)
}
