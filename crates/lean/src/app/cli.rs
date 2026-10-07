use std::path::PathBuf;

use anyhow::{Context as _, Result};

pub(crate) fn project_root_from_args() -> Result<PathBuf> {
    let argument = std::env::args_os().skip(1).find(|argument| {
        argument
            .to_str()
            .is_none_or(|argument| !argument.starts_with('-'))
    });
    let path = match argument {
        Some(argument) => PathBuf::from(argument),
        None => std::env::current_dir().context("reading the current directory")?,
    };
    let path = dunce::canonicalize(&path).with_context(|| format!("opening {}", path.display()))?;
    if path.is_dir() {
        Ok(path)
    } else {
        path.parent()
            .map(PathBuf::from)
            .with_context(|| format!("{} has no parent folder", path.display()))
    }
}
