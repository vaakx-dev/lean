use std::io::ErrorKind;
use std::path::PathBuf;

use gpui_kit::App;

use crate::storage::{QueuedWriter, state_file};

const FILE_NAME: &str = "recent-projects.json";

static WRITER: QueuedWriter = QueuedWriter::new();

pub(super) fn load() -> Vec<PathBuf> {
    let Some(file) = state_file(FILE_NAME) else {
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
    if let Some(file) = state_file(FILE_NAME) {
        WRITER.save(file, paths, cx);
    }
}
