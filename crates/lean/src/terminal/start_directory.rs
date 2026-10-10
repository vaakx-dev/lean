use std::path::{Path, PathBuf};

use gpui_kit::App;

use super::settings::{WORKING_DIRECTORY, WorkingDirectory};

pub(crate) struct StartDirectory {
    project_root: PathBuf,
    active_file: Box<dyn Fn(&App) -> Option<PathBuf>>,
}

impl StartDirectory {
    pub(crate) fn new(
        project_root: PathBuf,
        active_file: impl Fn(&App) -> Option<PathBuf> + 'static,
    ) -> Self {
        Self {
            project_root,
            active_file: Box::new(active_file),
        }
    }

    pub(super) fn resolve(&self, cx: &App) -> PathBuf {
        let preferred = match WORKING_DIRECTORY.get(cx) {
            WorkingDirectory::ProjectRoot => None,
            WorkingDirectory::ActiveFileFolder => (self.active_file)(cx)
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf),
            WorkingDirectory::Home => dirs::home_dir(),
        };
        preferred
            .filter(|directory| directory.is_dir())
            .unwrap_or_else(|| self.project_root.clone())
    }
}
