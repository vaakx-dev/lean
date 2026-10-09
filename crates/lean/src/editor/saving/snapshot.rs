use std::io;
use std::path::Path;

use super::super::disk_io::{DiskStamp, write_atomically};
use super::super::file_text::content_hash;

pub(super) struct Written {
    pub(super) stamp: DiskStamp,
    pub(super) hash: u64,
}

pub(super) fn write_snapshot(
    encode: Box<dyn FnOnce() -> Vec<u8> + Send>,
    path: &Path,
    recreate_folder: bool,
) -> io::Result<Written> {
    let bytes = encode();
    let hash = content_hash(&bytes);
    let stamp = write_atomically(path, &bytes, recreate_folder)?;
    Ok(Written { stamp, hash })
}
