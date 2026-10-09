use std::env;
use std::fs;
use std::path::PathBuf;

const ENVS_DIRS_KEY: &str = "envs_dirs:";
const HOME_FILES: &[&str] = &[
    ".condarc",
    ".conda/.condarc",
    ".conda/condarc",
    ".config/conda/.condarc",
    ".config/conda/condarc",
];

pub(super) fn envs_dirs() -> Vec<PathBuf> {
    let home = env::home_dir();
    let mut files: Vec<PathBuf> = env::var_os("CONDARC")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    if let Some(home) = &home {
        files.extend(HOME_FILES.iter().map(|file| home.join(file)));
    }
    files
        .iter()
        .filter_map(|file| fs::read_to_string(file).ok())
        .flat_map(|contents| list_values(&contents))
        .map(|dir| expand_home(&dir, home.as_ref()))
        .collect()
}

fn list_values(contents: &str) -> Vec<String> {
    contents
        .lines()
        .skip_while(|line| line.trim_end() != ENVS_DIRS_KEY)
        .skip(1)
        .map_while(|line| line.trim_start().strip_prefix("- "))
        .map(|value| value.trim().trim_matches(['"', '\'']).to_owned())
        .collect()
}

fn expand_home(dir: &str, home: Option<&PathBuf>) -> PathBuf {
    match (dir.strip_prefix("~/"), home) {
        (Some(rest), Some(home)) => home.join(rest),
        _ => PathBuf::from(dir),
    }
}
