use std::fs::{self, OpenOptions, Permissions};
use std::io::{self, Write as _};
use std::path::Path;

use serde::Serialize;

pub(crate) fn write_json_atomic<T: Serialize + ?Sized>(file: &Path, value: &T) -> io::Result<()> {
    let mut contents = serde_json::to_vec_pretty(value)?;
    contents.push(b'\n');
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(file, &contents)
}

pub(crate) fn write_atomic(file: &Path, bytes: &[u8]) -> io::Result<()> {
    let folder = file
        .parent()
        .ok_or_else(|| io::Error::other("the file has no parent folder"))?;
    let temporary = folder.join(temporary_name(file));
    let permissions = fs::metadata(file)
        .ok()
        .map(|metadata| metadata.permissions());
    let written =
        write_temporary(&temporary, bytes, permissions).and_then(|()| fs::rename(&temporary, file));
    if written.is_err()
        && let Err(error) = fs::remove_file(&temporary)
    {
        log::debug!("failed to remove {}: {error}", temporary.display());
    }
    written
}

fn write_temporary(
    temporary: &Path,
    bytes: &[u8],
    permissions: Option<Permissions>,
) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)?;
    file.write_all(bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()
}

fn temporary_name(file: &Path) -> String {
    let name = file
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    format!(".{name}.{}.lean-save", uuid::Uuid::new_v4().simple())
}
