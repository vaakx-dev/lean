use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use anyhow::Result;
use gpui_kit::{App, AppContext as _};

const APP_DIR: &str = "lean";
const FILE_NAME: &str = "recent-projects.json";

static LATEST_SAVE: AtomicU64 = AtomicU64::new(0);
static WRITING: Mutex<()> = Mutex::new(());

pub(super) fn load() -> Vec<PathBuf> {
    let Some(file) = file_path() else {
        return Vec::new();
    };
    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            log::warn!("failed to read {}: {error}", file.display());
            return Vec::new();
        }
    };
    match serde_json::from_str::<Vec<PathBuf>>(&contents) {
        Ok(paths) => paths,
        Err(error) => {
            log::warn!("failed to parse {}: {error}", file.display());
            Vec::new()
        }
    }
}

pub(super) fn save(paths: Vec<PathBuf>, cx: &App) {
    let generation = LATEST_SAVE.fetch_add(1, Ordering::SeqCst) + 1;
    cx.background_spawn(async move {
        let _writing = WRITING.lock().unwrap_or_else(PoisonError::into_inner);
        if LATEST_SAVE.load(Ordering::SeqCst) != generation {
            return;
        }
        let Some(file) = file_path() else {
            return;
        };
        if let Err(error) = write(&file, &paths) {
            log::warn!("failed to save {}: {error:#}", file.display());
        }
    })
    .detach();
}

fn write(file: &Path, paths: &[PathBuf]) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, serde_json::to_string_pretty(paths)?)?;
    Ok(())
}

fn file_path() -> Option<PathBuf> {
    let dir = dirs::state_dir().or_else(dirs::data_local_dir);
    if dir.is_none() {
        log::warn!("no state folder found; recent projects will not be saved");
    }
    dir.map(|dir| dir.join(APP_DIR).join(FILE_NAME))
}
