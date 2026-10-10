use std::path::PathBuf;

const APP_DIR: &str = "lean";

pub(crate) fn config_file(name: &str) -> Option<PathBuf> {
    let dir = dirs::config_dir();
    if dir.is_none() {
        log::warn!("no config folder found; {name} will not be saved");
    }
    dir.map(|dir| dir.join(APP_DIR).join(name))
}

pub(crate) fn state_file(name: &str) -> Option<PathBuf> {
    let dir = dirs::state_dir().or_else(dirs::data_local_dir);
    if dir.is_none() {
        log::warn!("no state folder found; {name} will not be saved");
    }
    dir.map(|dir| dir.join(APP_DIR).join(name))
}
