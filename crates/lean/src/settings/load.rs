use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use gpui_kit::SharedString;
use serde_json::{Map, Value};

const LEGACY_PLUGINS_KEY: &str = "plugins";

pub(super) fn load_initial(file: &Path) -> (Map<String, Value>, Option<SharedString>) {
    match read(file) {
        Ok(values) => (values, None),
        Err(error) => {
            log::warn!("failed to load {}: {error}", file.display());
            (Map::new(), Some(back_up(file, &error).into()))
        }
    }
}

pub(super) fn read(file: &Path) -> Result<Map<String, Value>, String> {
    let contents = match std::fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => return Err(error.to_string()),
    };
    if contents.trim().is_empty() {
        return Ok(Map::new());
    }
    let mut values: Map<String, Value> =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    migrate_plugins(&mut values);
    Ok(values)
}

fn migrate_plugins(values: &mut Map<String, Value>) {
    let Some(Value::Object(plugins)) = values.remove(LEGACY_PLUGINS_KEY) else {
        return;
    };
    for (id, enabled) in plugins {
        values.entry(format!("plugins.{id}")).or_insert(enabled);
    }
}

fn back_up(file: &Path, error: &str) -> String {
    let backup = backup_path(file);
    match std::fs::copy(file, &backup) {
        Ok(_) => format!(
            "settings.json could not be read ({error}). Lean is using default settings; a copy was saved to {}.",
            backup.display()
        ),
        Err(copy_error) => {
            log::warn!("failed to back up {}: {copy_error}", file.display());
            format!("settings.json could not be read ({error}). Lean is using default settings.")
        }
    }
}

fn backup_path(file: &Path) -> PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(".bak");
    PathBuf::from(name)
}
