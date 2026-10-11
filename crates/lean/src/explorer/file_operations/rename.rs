use std::io;
use std::path::{Path, PathBuf};

use gpui_kit::{Context, Window};

use super::report_failure;
use crate::explorer::Explorer;
use crate::plugins::PanelEvent;

impl Explorer {
    pub(super) fn rename_entry(
        &mut self,
        from: PathBuf,
        to: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (source, target) = (from.clone(), to.clone());
        self.run_disk_op(
            move || rename_on_disk(&source, &target),
            window,
            cx,
            move |explorer, result, window, cx| match result {
                Ok(()) => explorer.finish_rename(from, to, window, cx),
                Err(error) => report_failure("rename", &from, error, window, cx),
            },
        );
    }

    fn finish_rename(
        &mut self,
        from: PathBuf,
        to: PathBuf,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let moved_folders: Vec<PathBuf> = self
            .expanded
            .iter()
            .filter_map(|folder| moved_path(folder, &from, &to))
            .collect();
        self.expanded.retain(|folder| !folder.starts_with(&from));
        self.children.retain(|folder, _| !folder.starts_with(&from));
        self.expanded.extend(moved_folders.iter().cloned());
        for folder in moved_folders {
            self.load_directory(folder, window, cx);
        }
        if let Some(folder) = to.parent().map(Path::to_path_buf) {
            self.finish_file_operation(folder, Some(to.clone()), window, cx);
        }
        cx.emit(PanelEvent::PathRenamed { from, to });
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
    if to.symlink_metadata().is_ok() && !is_same_entry(from, to) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "a file or folder with this name already exists",
        ));
    }
    std::fs::rename(from, to)
}

fn is_same_entry(from: &Path, to: &Path) -> bool {
    let same_name = match (from.file_name(), to.file_name()) {
        (Some(from), Some(to)) => {
            from.to_string_lossy().to_lowercase() == to.to_string_lossy().to_lowercase()
        }
        _ => false,
    };
    same_name && from.parent() == to.parent() && is_same_file(from, to)
}

#[cfg(unix)]
fn is_same_file(from: &Path, to: &Path) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    match (from.symlink_metadata(), to.symlink_metadata()) {
        (Ok(from), Ok(to)) => from.dev() == to.dev() && from.ino() == to.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn is_same_file(from: &Path, to: &Path) -> bool {
    match (dunce::canonicalize(from), dunce::canonicalize(to)) {
        (Ok(from), Ok(to)) => from == to,
        _ => false,
    }
}
