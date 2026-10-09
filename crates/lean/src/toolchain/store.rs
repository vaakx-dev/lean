use std::collections::{BTreeMap, HashMap};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::Result;

const APP_DIR: &str = "lean";
const FILE_NAME: &str = "toolchains.json";

pub(super) type Choices = HashMap<String, Option<PathBuf>>;
type SavedChoices = BTreeMap<PathBuf, Choices>;

pub(super) fn load(project_root: &Path) -> Choices {
    read_all().remove(project_root).unwrap_or_default()
}

pub(super) fn save(project_root: &Path, choices: &Choices) {
    let Some(file) = file_path() else {
        return;
    };
    let mut saved = read_all();
    saved.insert(project_root.to_path_buf(), choices.clone());
    if let Err(error) = write(&file, &saved) {
        log::warn!("failed to save {}: {error:#}", file.display());
    }
}

fn read_all() -> SavedChoices {
    let Some(file) = file_path() else {
        return SavedChoices::new();
    };
    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return SavedChoices::new(),
        Err(error) => {
            log::warn!("failed to read {}: {error}", file.display());
            return SavedChoices::new();
        }
    };
    serde_json::from_str(&contents).unwrap_or_else(|error| {
        log::warn!("failed to parse {}: {error}", file.display());
        SavedChoices::new()
    })
}

fn write(file: &Path, saved: &SavedChoices) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, serde_json::to_string_pretty(saved)?)?;
    Ok(())
}

fn file_path() -> Option<PathBuf> {
    let dir = dirs::state_dir().or_else(dirs::data_local_dir);
    if dir.is_none() {
        log::warn!("no state folder found; toolchain choices will not be saved");
    }
    dir.map(|dir| dir.join(APP_DIR).join(FILE_NAME))
}
