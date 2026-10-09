mod conda;
mod pyenv;
mod venv;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::Toolchain;

const LANGUAGE: &str = "python";

#[cfg(not(windows))]
const PYTHON_EXECUTABLE: &str = "python";
#[cfg(windows)]
const PYTHON_EXECUTABLE: &str = "python.exe";

pub(super) fn discover(project_root: &Path) -> Vec<Toolchain> {
    let mut toolchains = venv::discover(project_root);
    toolchains.extend(conda::discover(project_root));
    toolchains.extend(pyenv::discover());
    toolchains
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path_string(path))
}

fn child_dirs(path: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn is_inherited(variable: &str, prefix: &Path) -> bool {
    env::var_os(variable)
        .and_then(|value| dunce::canonicalize(value).ok())
        .is_some_and(|value| value == prefix)
}
