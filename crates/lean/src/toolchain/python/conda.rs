#[cfg(feature = "terminal")]
mod activation;
mod condarc;
#[cfg(feature = "terminal")]
mod executable;
mod locations;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use gpui_kit::SharedString;

use super::{LANGUAGE, PYTHON_EXECUTABLE, folder_name, is_inherited, path_string};
use crate::toolchain::Toolchain;

const BASE_NAME: &str = "base";
const PREFIX_VARIABLE: &str = "CONDA_PREFIX";

#[cfg(not(windows))]
const BIN_DIRS: &[&str] = &["bin"];
#[cfg(windows)]
const BIN_DIRS: &[&str] = &[
    "",
    "Library\\mingw-w64\\bin",
    "Library\\usr\\bin",
    "Library\\bin",
    "Scripts",
    "bin",
];

#[cfg(not(windows))]
const PYTHON_DIR: &str = "bin";
#[cfg(windows)]
const PYTHON_DIR: &str = "";

pub(super) fn discover(project_root: &Path, searched: &[PathBuf]) -> Vec<Toolchain> {
    let mut seen = HashSet::new();
    locations::candidate_prefixes(project_root, searched)
        .into_iter()
        .filter_map(|prefix| dunce::canonicalize(prefix).ok())
        .filter(|prefix| seen.insert(prefix.clone()))
        .filter_map(read_environment)
        .collect()
}

fn read_environment(prefix: PathBuf) -> Option<Toolchain> {
    let metadata = prefix.join("conda-meta");
    if !metadata.is_dir() || !prefix.join(PYTHON_DIR).join(PYTHON_EXECUTABLE).exists() {
        return None;
    }
    let name = environment_name(&prefix);
    Some(Toolchain {
        language: LANGUAGE,
        name: name.clone().into(),
        kind: if locations::is_pixi(&prefix) {
            "pixi"
        } else {
            "conda"
        },
        version: python_version(&metadata),
        bin_dirs: BIN_DIRS.iter().map(|dir| prefix.join(dir)).collect(),
        variables: vec![
            (PREFIX_VARIABLE.to_owned(), path_string(&prefix)),
            ("CONDA_DEFAULT_ENV".to_owned(), name),
        ],
        #[cfg(feature = "terminal")]
        activation: executable::conda_executable(&prefix)
            .map(|conda| activation::activation(&conda, &prefix)),
        is_inherited: is_inherited(PREFIX_VARIABLE, &prefix),
        prefix,
    })
}

fn environment_name(prefix: &Path) -> String {
    if prefix.join("condabin").is_dir() {
        BASE_NAME.to_owned()
    } else {
        folder_name(prefix)
    }
}

fn python_version(metadata: &Path) -> Option<SharedString> {
    fs::read_dir(metadata).ok()?.find_map(|entry| {
        let file_name = entry.ok()?.file_name();
        let package = file_name.to_str()?.strip_suffix(".json")?;
        let version = package.strip_prefix("python-")?.split('-').next()?;
        version
            .starts_with(|character: char| character.is_ascii_digit())
            .then(|| version.to_owned().into())
    })
}
