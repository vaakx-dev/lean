use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use gpui_kit::App;

use super::settings::{STARTUP_OPEN, StartupFolder};
use crate::recent_projects;

pub(crate) struct LaunchTarget {
    folder: PathBuf,
    explicit: bool,
    pub(super) file: Option<PathBuf>,
}

pub(crate) fn launch_target_from_args() -> Result<LaunchTarget> {
    let Some(argument) = path_argument() else {
        let folder = std::env::current_dir().context("reading the current directory")?;
        return Ok(LaunchTarget {
            folder: canonical(&folder)?,
            explicit: false,
            file: None,
        });
    };
    let path = canonical(Path::new(&argument))?;
    if path.is_dir() {
        return Ok(LaunchTarget {
            folder: path,
            explicit: true,
            file: None,
        });
    }
    let folder = path
        .parent()
        .map(PathBuf::from)
        .with_context(|| format!("{} has no parent folder", path.display()))?;
    Ok(LaunchTarget {
        folder,
        explicit: true,
        file: Some(path),
    })
}

impl LaunchTarget {
    pub(super) fn project_root(&self, cx: &App) -> PathBuf {
        if !self.explicit
            && STARTUP_OPEN.get(cx) == StartupFolder::LastProject
            && let Some(last) = recent_projects::last_opened(cx)
        {
            return last;
        }
        self.folder.clone()
    }
}

fn path_argument() -> Option<OsString> {
    std::env::args_os().skip(1).find(|argument| {
        argument
            .to_str()
            .is_none_or(|argument| !argument.starts_with('-'))
    })
}

fn canonical(path: &Path) -> Result<PathBuf> {
    dunce::canonicalize(path).with_context(|| format!("opening {}", path.display()))
}
