use std::fs::{self, File};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result};
use serde::Serialize;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_json_atomic<T: Serialize + ?Sized>(file: &Path, value: &T) -> Result<()> {
    let mut contents = serde_json::to_vec_pretty(value)?;
    contents.push(b'\n');
    write_atomic(file, &contents)
}

fn write_atomic(file: &Path, contents: &[u8]) -> Result<()> {
    let parent = file
        .parent()
        .with_context(|| format!("{} has no parent folder", file.display()))?;
    fs::create_dir_all(parent)?;
    let temp = temp_path(file);
    let written = write_synced(&temp, contents).and_then(|()| Ok(fs::rename(&temp, file)?));
    if written.is_err() {
        fs::remove_file(&temp).ok();
    }
    written
}

fn write_synced(path: &Path, contents: &[u8]) -> Result<()> {
    let mut temp = File::create(path)?;
    temp.write_all(contents)?;
    temp.sync_all()?;
    Ok(())
}

fn temp_path(file: &Path) -> PathBuf {
    let name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    file.with_file_name(format!(".{name}.{}.{id}.tmp", std::process::id()))
}
