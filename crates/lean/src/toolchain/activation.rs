use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use super::Toolchain;

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
    clear_inherited_variables(known, &mut environment);
    environment
}

fn clear_inherited_variables(known: &[Toolchain], environment: &mut HashMap<String, String>) {
    for (name, _) in known.iter().flat_map(|toolchain| &toolchain.variables) {
        if env::var_os(name).is_some() {
            environment.entry(name.clone()).or_default();
        }
    }
}
