use std::env;
use std::path::{Path, PathBuf};

use super::venv::config_version;
use super::{LANGUAGE, PYTHON_EXECUTABLE, child_dirs, folder_name};
use crate::toolchain::Toolchain;

const VERSION_VARIABLE: &str = "PYENV_VERSION";

#[cfg(not(windows))]
const DEFAULT_ROOT: &str = ".pyenv";
#[cfg(windows)]
const DEFAULT_ROOT: &str = ".pyenv\\pyenv-win";

pub(super) fn discover(root: Option<&Path>) -> Vec<Toolchain> {
    let Some(root) = root.map(Path::to_path_buf).or_else(default_root) else {
        return Vec::new();
    };
    child_dirs(&root.join("versions"))
        .into_iter()
        .filter_map(read_version)
        .collect()
}

fn default_root() -> Option<PathBuf> {
    env::var_os("PYENV_ROOT")
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|home| home.join(DEFAULT_ROOT)))
}

fn read_version(path: PathBuf) -> Option<Toolchain> {
    let prefix = dunce::canonicalize(&path).ok()?;
    let bin_dirs = bin_dirs(&prefix);
    if !bin_dirs[0].join(PYTHON_EXECUTABLE).exists() {
        return None;
    }
    let name = folder_name(&path);
    Some(Toolchain {
        language: LANGUAGE,
        kind: "pyenv",
        version: config_version(&prefix),
        is_inherited: env::var(VERSION_VARIABLE).is_ok_and(|version| version == name),
        variables: vec![(VERSION_VARIABLE.to_owned(), name.clone())],
        name: name.into(),
        #[cfg(feature = "terminal")]
        activation: None,
        bin_dirs,
        prefix,
    })
}

#[cfg(not(windows))]
fn bin_dirs(prefix: &Path) -> Vec<PathBuf> {
    vec![prefix.join("bin")]
}

#[cfg(windows)]
fn bin_dirs(prefix: &Path) -> Vec<PathBuf> {
    vec![prefix.to_path_buf(), prefix.join("Scripts")]
}
