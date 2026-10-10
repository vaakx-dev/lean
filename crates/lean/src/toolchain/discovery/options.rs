use std::env;
use std::path::{Path, PathBuf};

use gpui_kit::App;

use crate::toolchain::settings::{
    MAX_VENV_SEARCH_DEPTH, PYENV_ROOT, SEARCH_PATHS, VENV_SEARCH_DEPTH,
};

pub(crate) struct DiscoveryOptions {
    pub(crate) venv_depth: u32,
    pub(crate) search_paths: Vec<PathBuf>,
    pub(crate) pyenv_root: Option<PathBuf>,
}

impl DiscoveryOptions {
    pub(crate) fn from_settings(project_root: &Path, cx: &App) -> Self {
        Self {
            venv_depth: VENV_SEARCH_DEPTH.get(cx).min(MAX_VENV_SEARCH_DEPTH),
            search_paths: SEARCH_PATHS
                .get(cx)
                .iter()
                .map(|path| resolve(path, project_root))
                .collect(),
            pyenv_root: PYENV_ROOT.get(cx).map(|path| resolve(&path, project_root)),
        }
    }
}

fn resolve(path: &str, project_root: &Path) -> PathBuf {
    project_root.join(expand_home(path))
}

fn expand_home(path: &str) -> PathBuf {
    match (path.strip_prefix('~'), env::home_dir()) {
        (Some(""), Some(home)) => home,
        (Some(rest), Some(home)) if rest.starts_with(['/', '\\']) => {
            home.join(rest.trim_start_matches(['/', '\\']))
        }
        _ => PathBuf::from(path),
    }
}
