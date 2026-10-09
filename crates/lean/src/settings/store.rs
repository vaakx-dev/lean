use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use anyhow::Result;
use gpui_kit::{App, AppContext as _};
use serde_json::{Map, Value};

const APP_DIR: &str = "lean";
const FILE_NAME: &str = "settings.json";

static LATEST_SAVE: AtomicU64 = AtomicU64::new(0);
static WRITING: Mutex<()> = Mutex::new(());

pub(super) fn load() -> Map<String, Value> {
    let Some(file) = file_path() else {
        return Map::new();
    };
    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Map::new(),
        Err(error) => {
            log::warn!("failed to read {}: {error}", file.display());
            return Map::new();
        }
    };
    serde_json::from_str(&contents).unwrap_or_else(|error| {
        log::warn!("failed to parse {}: {error}", file.display());
        Map::new()
    })
}

pub(super) fn save(document: Map<String, Value>, cx: &App) {
    let generation = LATEST_SAVE.fetch_add(1, Ordering::SeqCst) + 1;
    cx.background_spawn(async move {
        let _writing = WRITING.lock().unwrap_or_else(PoisonError::into_inner);
        if LATEST_SAVE.load(Ordering::SeqCst) != generation {
            return;
        }
        let Some(file) = file_path() else {
            return;
        };
        if let Err(error) = write(&file, &document) {
            log::warn!("failed to save {}: {error:#}", file.display());
        }
    })
    .detach();
}

fn write(file: &Path, document: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, serde_json::to_string_pretty(document)?)?;
    Ok(())
}

fn file_path() -> Option<PathBuf> {
    let dir = dirs::config_dir();
    if dir.is_none() {
        log::warn!("no config folder found; settings will not be saved");
    }
    dir.map(|dir| dir.join(APP_DIR).join(FILE_NAME))
}
