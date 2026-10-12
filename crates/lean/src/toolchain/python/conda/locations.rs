use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::condarc;
use crate::toolchain::python::child_dirs;

const PIXI_DIR: &str = ".pixi";
const HOME_INSTALL_DIRS: &[&str] = &[
    "anaconda3",
    "miniconda3",
    "miniforge3",
    "mambaforge",
    "micromamba",
    ".local/share/mamba",
];

#[cfg(not(windows))]
const SYSTEM_INSTALL_DIRS: &[&str] = &[
    "/opt/conda",
    "/opt/anaconda3",
    "/opt/miniconda3",
    "/opt/miniforge3",
];
#[cfg(windows)]
const SYSTEM_INSTALL_DIRS: &[&str] = &[
    "C:\\ProgramData\\anaconda3",
    "C:\\ProgramData\\miniconda3",
    "C:\\ProgramData\\miniforge3",
];

pub(super) fn candidate_prefixes(project_root: &Path, searched: &[PathBuf]) -> Vec<PathBuf> {
    let mut listed = HashSet::new();
    let mut envs_in = |container: PathBuf| {
        if listed.insert(container.clone()) {
            child_dirs(&container)
        } else {
            Vec::new()
        }
    };
    let mut prefixes = envs_in(project_root.join(PIXI_DIR).join("envs"));
    prefixes.extend(searched.iter().cloned());
    for root in installation_roots() {
        let envs = envs_in(root.join("envs"));
        prefixes.push(root);
        prefixes.extend(envs);
    }
    for dir in condarc::envs_dirs() {
        prefixes.extend(envs_in(dir));
    }
    if let Some(prefix) = env::var_os("CONDA_PREFIX") {
        prefixes.push(PathBuf::from(prefix));
    }
    if let Some(home) = env::home_dir() {
        let conda_dir = home.join(".conda");
        prefixes.extend(listed_environments(&conda_dir.join("environments.txt")));
        prefixes.extend(envs_in(conda_dir.join("envs")));
    }
    prefixes
}

pub(super) fn is_pixi(prefix: &Path) -> bool {
    prefix
        .parent()
        .and_then(Path::parent)
        .is_some_and(|dir| dir.ends_with(PIXI_DIR))
}

fn installation_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(conda_exe) = env::var_os("CONDA_EXE")
        && let Some(root) = Path::new(&conda_exe).parent().and_then(Path::parent)
    {
        roots.push(root.to_path_buf());
    }
    if let Some(home) = env::home_dir() {
        roots.extend(HOME_INSTALL_DIRS.iter().map(|dir| home.join(dir)));
    }
    roots.extend(SYSTEM_INSTALL_DIRS.iter().map(PathBuf::from));
    roots
}

fn listed_environments(path: &Path) -> Vec<PathBuf> {
    fs::read_to_string(path)
        .map(|contents| {
            contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}
