use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::toolchain::Toolchain;

const CONDA_METADATA: &str = "conda-meta";

pub(super) fn watched_folders(
    project_root: &Path,
    search_paths: &[PathBuf],
    toolchains: &[Toolchain],
) -> Vec<PathBuf> {
    let mut folders = BTreeSet::from([project_root.to_path_buf()]);
    folders.extend(search_paths.iter().cloned());
    for toolchain in toolchains {
        folders.insert(toolchain.prefix.clone());
        folders.insert(toolchain.prefix.join(CONDA_METADATA));
        folders.extend(toolchain.prefix.parent().map(Path::to_path_buf));
    }
    folders
        .into_iter()
        .filter(|folder| folder.is_dir())
        .collect()
}
