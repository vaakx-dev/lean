use std::fmt;
use std::fs::{self, Metadata, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::file_text::content_hash;

const MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;
const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct DiskStamp {
    modified: Option<SystemTime>,
    len: u64,
}

impl DiskStamp {
    fn of(metadata: &Metadata) -> Self {
        Self {
            modified: metadata.modified().ok(),
            len: metadata.len(),
        }
    }
}

pub(super) struct DiskFile {
    pub(super) bytes: Vec<u8>,
    pub(super) hash: u64,
    pub(super) stamp: DiskStamp,
}

pub(super) enum ReadError {
    NotFound,
    PermissionDenied,
    IsFolder,
    TooLarge(u64),
    Io(io::Error),
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound,
            io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            io::ErrorKind::IsADirectory => Self::IsFolder,
            _ => Self::Io(error),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "the file doesn't exist"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::IsFolder => write!(f, "it is a folder"),
            Self::TooLarge(size) => write!(
                f,
                "the file is {} MB, larger than the {} MB limit",
                size / BYTES_PER_MEGABYTE,
                MAX_FILE_SIZE / BYTES_PER_MEGABYTE
            ),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

pub(super) fn read_file(path: &Path) -> Result<DiskFile, ReadError> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        return Err(ReadError::IsFolder);
    }
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ReadError::TooLarge(metadata.len()));
    }
    let bytes = fs::read(path)?;
    let hash = content_hash(&bytes);
    Ok(DiskFile {
        bytes,
        hash,
        stamp: DiskStamp::of(&metadata),
    })
}

pub(super) fn read_changed_file(
    path: &Path,
    known: Option<DiskStamp>,
) -> Result<Option<DiskFile>, ReadError> {
    let is_unchanged = known.is_some_and(|known| {
        fs::metadata(path).is_ok_and(|metadata| DiskStamp::of(&metadata) == known)
    });
    if is_unchanged {
        return Ok(None);
    }
    read_file(path).map(Some)
}

pub(super) fn write_atomically(
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
    match replace_file(&target, bytes, original.as_ref()) {
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            log::debug!("writing {} in place: {error}", target.display());
            fs::write(&target, bytes)?;
        }
        result => result?,
    }
    fs::metadata(&target).map(|metadata| DiskStamp::of(&metadata))
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
