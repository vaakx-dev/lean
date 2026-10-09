use std::env;
use std::fs::File;
use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};

const HISTORY_COMMAND: &str = "# cmd: ";

#[cfg(not(windows))]
const EXECUTABLES: &[&str] = &["bin/conda", "condabin/conda"];
#[cfg(windows)]
const EXECUTABLES: &[&str] = &["Scripts\\conda.exe", "condabin\\conda.bat"];

pub(super) fn conda_executable(prefix: &Path) -> Option<PathBuf> {
    executable_in(prefix)
        .or_else(|| envs_root(prefix).and_then(executable_in))
        .or_else(|| history_root(prefix).as_deref().and_then(executable_in))
        .or_else(inherited_executable)
}

fn executable_in(root: &Path) -> Option<PathBuf> {
    EXECUTABLES
        .iter()
        .map(|executable| root.join(executable))
        .find(|path| path.is_file())
}

fn envs_root(prefix: &Path) -> Option<&Path> {
    prefix
        .parent()
        .filter(|parent| parent.ends_with("envs"))
        .and_then(Path::parent)
}

fn history_root(prefix: &Path) -> Option<PathBuf> {
    let history = File::open(prefix.join("conda-meta").join("history")).ok()?;
    BufReader::new(history)
        .lines()
        .map_while(Result::ok)
        .find_map(|line| {
            let command = line.strip_prefix(HISTORY_COMMAND)?;
            let program = Path::new(command.split_whitespace().next()?);
            program.parent()?.parent().map(Path::to_path_buf)
        })
}

fn inherited_executable() -> Option<PathBuf> {
    env::var_os("CONDA_EXE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}
