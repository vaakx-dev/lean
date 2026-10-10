use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use super::DiskStamp;

pub(in crate::editor) fn write_atomically(
    path: &Path,
    bytes: &[u8],
    recreate_folder: bool,
) -> io::Result<DiskStamp> {
    if recreate_folder && let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let target = resolve_symlink(path);
    let original = fs::metadata(&target).ok();
    if original
        .as_ref()
        .is_some_and(|metadata| metadata.permissions().readonly())
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file is read-only",
        ));
    }
    replace_file(&target, bytes, original.as_ref()).map_err(refuse_in_place)?;
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

fn replace_file(target: &Path, bytes: &[u8], original: Option<&Metadata>) -> io::Result<()> {
    let folder = target
        .parent()
        .ok_or_else(|| io::Error::other("the file has no parent folder"))?;
    let temporary = folder.join(temporary_name(target));
    let written =
        write_temporary(&temporary, bytes, original).and_then(|()| fs::rename(&temporary, target));
    if written.is_err()
        && let Err(error) = fs::remove_file(&temporary)
    {
        log::debug!("failed to remove {}: {error}", temporary.display());
    }
    written
}

fn write_temporary(temporary: &Path, bytes: &[u8], original: Option<&Metadata>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)?;
    file.write_all(bytes)?;
    if let Some(original) = original {
        file.set_permissions(original.permissions())?;
    }
    file.sync_all()
}

fn temporary_name(target: &Path) -> String {
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    format!(".{name}.{}.lean-save", uuid::Uuid::new_v4().simple())
}
