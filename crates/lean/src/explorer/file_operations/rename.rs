use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AppContext as _, Context, Window};

use crate::explorer::{Explorer, ExplorerEvent};
use crate::files::file_name;

impl Explorer {
    pub(super) fn rename_entry(
        &mut self,
        from: PathBuf,
        to: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let task = cx.background_spawn({
            let from = from.clone();
            let to = to.clone();
            async move { rename_on_disk(&from, &to) }
        });
        cx.spawn_in(window, async move |explorer, cx| {
            let result = task.await;
            let update = explorer.update_in(cx, |explorer, window, cx| match result {
                Ok(()) => explorer.finish_rename(from, to, cx),
                Err(error) => {
                    log::warn!("failed to rename {}: {error}", from.display());
                    window.push_notification(
                        format!("Can't rename \"{}\": {error}", file_name(&from)),
                        cx,
                    );
                }
            });
            if let Err(error) = update {
                log::debug!("explorer dropped while renaming an entry: {error:#}");
            }
        })
        .detach();
    }

    fn finish_rename(&mut self, from: PathBuf, to: PathBuf, cx: &mut Context<Self>) {
        let moved_folders: Vec<PathBuf> = self
            .expanded
            .iter()
            .filter_map(|folder| moved_path(folder, &from, &to))
            .collect();
        self.expanded.retain(|folder| !folder.starts_with(&from));
        self.children.retain(|folder, _| !folder.starts_with(&from));
        self.expanded.extend(moved_folders.iter().cloned());
        for folder in moved_folders {
            self.load_directory(folder, cx);
        }
        if let Some(folder) = to.parent().map(Path::to_path_buf) {
            self.finish_file_operation(folder, Some(to.clone()), cx);
        }
        cx.emit(ExplorerEvent::PathRenamed { from, to });
    }
}

fn moved_path(path: &Path, from: &Path, to: &Path) -> Option<PathBuf> {
    let suffix = path.strip_prefix(from).ok()?;
    if suffix.as_os_str().is_empty() {
        Some(to.to_path_buf())
    } else {
        Some(to.join(suffix))
    }
}

fn rename_on_disk(from: &Path, to: &Path) -> io::Result<()> {
    let is_case_change =
        from.to_string_lossy().to_lowercase() == to.to_string_lossy().to_lowercase();
    if !is_case_change && to.symlink_metadata().is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "a file or folder with this name already exists",
        ));
    }
    std::fs::rename(from, to)
}
