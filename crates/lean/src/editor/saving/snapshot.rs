use std::io;
use std::path::Path;

use super::super::disk_io::{DiskStamp, ReadError, read_changed_file, write_atomically};
use super::super::file_text::content_hash;

pub(super) struct ExpectedDisk {
    pub(super) stamp: Option<DiskStamp>,
    pub(super) hash: u64,
}

pub(super) enum DiskWrite {
    Written { stamp: DiskStamp, hash: u64 },
    ChangedOnDisk,
}

pub(super) fn write_snapshot(
    encode: Box<dyn FnOnce() -> Vec<u8> + Send>,
    path: &Path,
    recreate_folder: bool,
    expected: Option<ExpectedDisk>,
) -> io::Result<DiskWrite> {
    if let Some(expected) = expected
        && changed_on_disk(path, &expected)
    {
        return Ok(DiskWrite::ChangedOnDisk);
    }
    let bytes = encode();
    let hash = content_hash(&bytes);
    let stamp = write_atomically(path, &bytes, recreate_folder)?;
    Ok(DiskWrite::Written { stamp, hash })
}

fn changed_on_disk(path: &Path, expected: &ExpectedDisk) -> bool {
    match read_changed_file(path, expected.stamp) {
        Ok(None) | Err(ReadError::NotFound) => false,
        Ok(Some(disk)) => disk.hash != expected.hash,
        Err(error) => {
            log::debug!("can't check {} before saving: {error}", path.display());
            true
        }
    }
}
