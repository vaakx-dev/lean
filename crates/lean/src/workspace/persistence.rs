mod session;
mod session_store;
mod window_state;

use std::io::ErrorKind;

use serde::de::DeserializeOwned;

use crate::storage::state_file;

pub(crate) use window_state::restored_window_bounds;

fn read_state<T: DeserializeOwned>(name: &str) -> Option<T> {
    let file = state_file(name)?;
    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return None,
        Err(error) => {
            log::warn!("failed to read {}: {error}", file.display());
            return None;
        }
    };
    serde_json::from_str(&contents)
        .inspect_err(|error| log::warn!("failed to parse {}: {error}", file.display()))
        .ok()
}
