mod atomic_write;
mod queued_writer;
mod read_state;

use std::path::PathBuf;

pub(crate) use atomic_write::{write_atomic, write_json_atomic};
pub(crate) use queued_writer::QueuedWriter;
pub(crate) use read_state::read_state;

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
