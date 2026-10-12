use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use gpui_kit::SharedString;

use super::{LANGUAGE, PYTHON_EXECUTABLE, child_dirs, folder_name, is_inherited, path_string};
use crate::toolchain::Toolchain;
#[cfg(feature = "terminal")]
use crate::toolchain::shell_activation::{
    ShellActivation, fish_quote, posix_quote, powershell_quote,
};

const CONFIG_FILE: &str = "pyvenv.cfg";
const PREFIX_VARIABLE: &str = "VIRTUAL_ENV";
const SKIPPED_FOLDERS: &[&str] = &["node_modules", ".git", "target", "__pycache__"];

#[cfg(not(windows))]
const BIN_DIR: &str = "bin";
#[cfg(windows)]
const BIN_DIR: &str = "Scripts";

pub(super) fn discover(root: &Path, depth: u32) -> Vec<Toolchain> {
    let mut toolchains = Vec::new();
    collect(root, depth, &mut toolchains);
    sorted_by_name(toolchains)
}

pub(super) fn read_all(folders: &[PathBuf]) -> Vec<Toolchain> {
    sorted_by_name(
        folders
            .iter()
            .filter_map(|folder| read_venv(folder))
            .collect(),
    )
}

pub(super) fn discover_inherited() -> Option<Toolchain> {
    env::var_os(PREFIX_VARIABLE).and_then(|prefix| read_venv(Path::new(&prefix)))
}

pub(super) fn config_version(prefix: &Path) -> Option<SharedString> {
    let config = fs::read_to_string(prefix.join(CONFIG_FILE)).ok()?;
    config.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        matches!(key.trim(), "version" | "version_info").then(|| value.trim().to_owned().into())
    })
}

fn collect(folder: &Path, depth: u32, found: &mut Vec<Toolchain>) {
    if let Some(toolchain) = read_venv(folder) {
        found.push(toolchain);
        return;
    }
    if depth == 0 {
        return;
    }
    for child in child_dirs(folder).iter().filter(|child| !is_skipped(child)) {
        collect(child, depth - 1, found);
    }
}

fn sorted_by_name(mut toolchains: Vec<Toolchain>) -> Vec<Toolchain> {
    toolchains.sort_by(|left, right| left.name.cmp(&right.name));
    toolchains
}

fn is_skipped(folder: &Path) -> bool {
    folder
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| SKIPPED_FOLDERS.contains(&name))
}

fn read_venv(path: &Path) -> Option<Toolchain> {
    if !path.join(CONFIG_FILE).is_file() {
        return None;
    }
    let prefix = dunce::canonicalize(path).ok()?;
    let bin_dir = prefix.join(BIN_DIR);
    if !bin_dir.join(PYTHON_EXECUTABLE).exists() {
        return None;
    }
    Some(Toolchain {
        language: LANGUAGE,
        name: folder_name(&prefix).into(),
        kind: "venv",
        version: config_version(&prefix),
        variables: vec![(PREFIX_VARIABLE.to_owned(), path_string(&prefix))],
        #[cfg(feature = "terminal")]
        activation: Some(activation(&bin_dir)),
        is_inherited: is_inherited(PREFIX_VARIABLE, &prefix),
        bin_dirs: vec![bin_dir],
        prefix,
    })
}

#[cfg(feature = "terminal")]
fn activation(bin_dir: &Path) -> ShellActivation {
    let script = |name: &str| path_string(&bin_dir.join(name));
    ShellActivation {
        posix: format!(". {}", posix_quote(&script("activate"))),
        fish: format!("source {}", fish_quote(&script("activate.fish"))),
        powershell: format!("& {}", powershell_quote(&script("Activate.ps1"))),
    }
}
