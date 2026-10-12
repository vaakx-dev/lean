mod conda;
mod pyenv;
mod venv;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gpui_kit::App;

use super::Toolchain;
use super::settings::{MAX_VENV_SEARCH_DEPTH, PYENV_ROOT, SEARCH_PATHS, VENV_SEARCH_DEPTH};

const LANGUAGE: &str = "python";

#[cfg(not(windows))]
const PYTHON_EXECUTABLE: &str = "python";
#[cfg(windows)]
const PYTHON_EXECUTABLE: &str = "python.exe";

pub(super) struct DiscoveryOptions {
    venv_depth: u32,
    pub(super) search_paths: Vec<PathBuf>,
    pyenv_root: Option<PathBuf>,
}

impl DiscoveryOptions {
    pub(super) fn from_settings(project_root: &Path, cx: &App) -> Self {
        Self {
            venv_depth: VENV_SEARCH_DEPTH.get(cx).min(MAX_VENV_SEARCH_DEPTH),
            search_paths: SEARCH_PATHS
                .get(cx)
                .iter()
                .map(|path| resolve(path, project_root))
                .collect(),
            pyenv_root: PYENV_ROOT.get(cx).map(|path| resolve(&path, project_root)),
        }
    }
}

pub(super) fn discover(project_root: &Path, options: &DiscoveryOptions) -> Vec<Toolchain> {
    let searched = search_candidates(&options.search_paths);
    let mut toolchains = venv::discover(project_root, options.venv_depth);
    toolchains.extend(venv::discover_inherited());
    toolchains.extend(venv::read_all(&searched));
    toolchains.extend(conda::discover(project_root, &searched));
    toolchains.extend(pyenv::discover(options.pyenv_root.as_deref()));
    let mut seen = HashSet::new();
    toolchains.retain(|toolchain| seen.insert(toolchain.prefix.clone()));
    toolchains
}

fn search_candidates(search_paths: &[PathBuf]) -> Vec<PathBuf> {
    search_paths
        .iter()
        .flat_map(|folder| {
            let mut candidates = vec![folder.clone()];
            candidates.extend(child_dirs(folder));
            candidates.extend(child_dirs(&folder.join("envs")));
            candidates
        })
        .collect()
}

fn resolve(path: &str, project_root: &Path) -> PathBuf {
    project_root.join(expand_home(path))
}

fn expand_home(path: &str) -> PathBuf {
    match (path.strip_prefix('~'), env::home_dir()) {
        (Some(""), Some(home)) => home,
        (Some(rest), Some(home)) if rest.starts_with(['/', '\\']) => {
            home.join(rest.trim_start_matches(['/', '\\']))
        }
        _ => PathBuf::from(path),
    }
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
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            let path = entry.path();
            (file_type.is_dir() || (file_type.is_symlink() && path.is_dir())).then_some(path)
        })
        .collect();
    dirs.sort();
    dirs
}

fn is_inherited(variable: &str, prefix: &Path) -> bool {
    env::var_os(variable)
        .and_then(|value| dunce::canonicalize(value).ok())
        .is_some_and(|value| value == prefix)
}
