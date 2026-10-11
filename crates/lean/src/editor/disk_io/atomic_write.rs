use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::DiskStamp;
use crate::storage::write_atomic;

pub(in crate::editor) fn write_atomically(
    path: &Path,
    bytes: &[u8],
    recreate_folder: bool,
) -> io::Result<DiskStamp> {
    if recreate_folder && let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let target = resolve_symlink(path);
    if fs::metadata(&target).is_ok_and(|metadata| metadata.permissions().readonly()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file is read-only",
        ));
    }
    write_atomic(&target, bytes).map_err(refuse_in_place)?;
    fs::metadata(&target).map(|metadata| DiskStamp::of(&metadata))
}

fn refuse_in_place(error: io::Error) -> io::Error {
    if error.kind() != io::ErrorKind::PermissionDenied {
        return error;
    }
    io::Error::new(
        error.kind(),
        format!(
            "can't create a temporary file in its folder ({error}); \
             the file was left unchanged rather than overwritten in place"
        ),
    )
}

fn resolve_symlink(path: &Path) -> PathBuf {
    let is_symlink =
        fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink());
    if !is_symlink {
        return path.to_path_buf();
    }
    dunce::canonicalize(path).unwrap_or_else(|error| {
        log::debug!("failed to resolve {}: {error}", path.display());
        path.to_path_buf()
    })
}
