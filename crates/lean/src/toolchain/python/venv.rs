use std::fs;
use std::path::Path;

use gpui_kit::SharedString;

use super::{LANGUAGE, PYTHON_EXECUTABLE, folder_name, is_inherited, path_string};
use crate::toolchain::Toolchain;
use crate::toolchain::shell_activation::{
    ShellActivation, fish_quote, posix_quote, powershell_quote,
};

const CONFIG_FILE: &str = "pyvenv.cfg";
const PREFIX_VARIABLE: &str = "VIRTUAL_ENV";

#[cfg(not(windows))]
const BIN_DIR: &str = "bin";
#[cfg(windows)]
const BIN_DIR: &str = "Scripts";

pub(super) fn discover(project_root: &Path) -> Vec<Toolchain> {
    let entries = match fs::read_dir(project_root) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!(
                "failed to scan {} for venvs: {error}",
                project_root.display()
            );
            return Vec::new();
        }
    };
    let mut toolchains: Vec<Toolchain> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| read_venv(&entry.path()))
        .collect();
    toolchains.sort_by(|left, right| left.name.cmp(&right.name));
    toolchains
}

pub(super) fn config_version(prefix: &Path) -> Option<SharedString> {
    let config = fs::read_to_string(prefix.join(CONFIG_FILE)).ok()?;
    config.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        matches!(key.trim(), "version" | "version_info").then(|| value.trim().to_owned().into())
    })
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
        activation: Some(activation(&bin_dir)),
        is_inherited: is_inherited(PREFIX_VARIABLE, &prefix),
        bin_dirs: vec![bin_dir],
        prefix,
    })
}

fn activation(bin_dir: &Path) -> ShellActivation {
    let script = |name: &str| path_string(&bin_dir.join(name));
    ShellActivation {
        posix: format!(". {}", posix_quote(&script("activate"))),
        fish: format!("source {}", fish_quote(&script("activate.fish"))),
        powershell: format!("& {}", powershell_quote(&script("Activate.ps1"))),
    }
}
