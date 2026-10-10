mod atomic_write;
mod read_error;

use std::fs::{self, Metadata};
use std::path::Path;
use std::time::SystemTime;

use super::file_text::content_hash;
use crate::files::settings::max_file_bytes;

pub(super) use atomic_write::write_atomically;
pub(super) use read_error::ReadError;

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

pub(super) fn read_file(path: &Path) -> Result<DiskFile, ReadError> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        return Err(ReadError::IsFolder);
    }
    let limit = max_file_bytes();
    if metadata.len() > limit {
        return Err(ReadError::TooLarge {
            size: metadata.len(),
            limit,
        });
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
