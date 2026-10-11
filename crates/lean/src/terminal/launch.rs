use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gpui_kit::App;

use super::settings::{WORKING_DIRECTORY, WorkingDirectory};

#[derive(Default)]
pub(crate) struct TerminalLaunch {
    pub(crate) active_file: Option<PathBuf>,
    pub(crate) environment: HashMap<String, String>,
    pub(crate) startup_command: Option<String>,
}

impl TerminalLaunch {
    pub(super) fn working_directory(&self, project_root: &Path, cx: &App) -> PathBuf {
        let preferred = match WORKING_DIRECTORY.get(cx) {
            WorkingDirectory::ProjectRoot => None,
            WorkingDirectory::ActiveFileFolder => self
                .active_file
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
            WorkingDirectory::Home => dirs::home_dir(),
        };
        preferred
            .filter(|directory| directory.is_dir())
            .unwrap_or_else(|| project_root.to_path_buf())
    }
}
