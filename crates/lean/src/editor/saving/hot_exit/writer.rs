use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use gpui_kit::App;

use super::store::BackupFile;
use crate::storage::write_json_atomic;

pub(super) type BackupChange = (PathBuf, Option<BackupFile>);

static PENDING: Mutex<BTreeMap<PathBuf, Option<BackupFile>>> = Mutex::new(BTreeMap::new());
static WRITING: Mutex<()> = Mutex::new(());

pub(super) fn write_later(changes: Vec<BackupChange>, cx: &App) {
    queue(changes);
    cx.background_executor().spawn(async { flush() }).detach();
}

pub(super) fn write_now(changes: Vec<BackupChange>) {
    queue(changes);
    flush();
}

fn queue(changes: Vec<BackupChange>) {
    lock(&PENDING).extend(changes);
}

fn flush() {
    let _writing = lock(&WRITING);
    let pending = std::mem::take(&mut *lock(&PENDING));
    for (file, backup) in pending {
        let written = match backup {
            Some(backup) => write_json_atomic(&file, &backup),
            None => remove(&file),
        };
        if let Err(error) = written {
            log::warn!("failed to update {}: {error}", file.display());
        }
    }
}

fn remove(file: &Path) -> io::Result<()> {
    match fs::remove_file(file) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
