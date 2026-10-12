use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::toolchain::Toolchain;

const CONTAINERS: &[&str] = &["envs", "versions"];

pub(in crate::toolchain) fn watched_folders(
    project_root: &Path,
    search_paths: &[PathBuf],
    toolchains: &[Toolchain],
) -> Vec<PathBuf> {
    let containers = toolchains
        .iter()
        .filter_map(|toolchain| toolchain.prefix.parent())
        .filter(|parent| is_container(parent))
        .map(Path::to_path_buf);
    let mut folders = BTreeSet::from([project_root.to_path_buf()]);
    folders.extend(search_paths.iter().cloned());
    folders.extend(containers);
    folders
        .into_iter()
        .filter(|folder| folder.is_dir())
        .collect()
}

fn is_container(folder: &Path) -> bool {
    folder
        .file_name()
        .is_some_and(|name| CONTAINERS.iter().any(|container| name == *container))
}
