use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use super::Toolchain;
use crate::terminal::ShellKind;

pub(super) fn activation_environment(
    known: &[Toolchain],
    active: &[&Toolchain],
) -> HashMap<String, String> {
    let mut environment = HashMap::new();
    if known.is_empty() {
        return environment;
    }
    let known_dirs: Vec<&Path> = known
        .iter()
        .flat_map(|toolchain| toolchain.bin_dirs.iter().map(PathBuf::as_path))
        .collect();
    let inherited: Vec<PathBuf> = env::var_os("PATH")
        .map(|path| {
            env::split_paths(&path)
                .filter(|dir| !known_dirs.contains(&dir.as_path()))
                .collect()
        })
        .unwrap_or_default();
    let paths = active
        .iter()
        .flat_map(|toolchain| toolchain.bin_dirs.iter().cloned())
        .chain(inherited);
    match env::join_paths(paths) {
        Ok(path) => {
            environment.insert("PATH".to_owned(), path.to_string_lossy().into_owned());
        }
        Err(error) => log::warn!("failed to build PATH for the active toolchains: {error}"),
    }
    for toolchain in active {
        environment.extend(toolchain.variables.iter().cloned());
    }
    environment
}

pub(super) fn activation_command(active: &[&Toolchain], shell: ShellKind) -> Option<String> {
    let commands: Vec<&str> = active
        .iter()
        .filter_map(|toolchain| toolchain.activation.as_ref()?.command(shell))
        .collect();
    (!commands.is_empty()).then(|| format!(" {}", commands.join("; ")))
}
