use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::App;
use notify::Event;
use serde_json::{Map, Value};

use super::global::{Settings, replace_all};
use super::{WRITER, load};
use crate::watcher::{is_noise, reload_on_change};

const RELOAD_DELAY: Duration = Duration::from_millis(150);

type Loaded = Option<Result<Map<String, Value>, String>>;

pub(super) fn start(file: PathBuf, cx: &mut App) {
    let Some(folder) = file.parent().map(Path::to_path_buf) else {
        return;
    };
    let Some(name) = file.file_name().map(OsString::from) else {
        return;
    };
    let is_settings_change = move |event: &Event| {
        !is_noise(event.kind)
            && event
                .paths
                .iter()
                .any(|path| path.file_name() == Some(name.as_os_str()))
    };
    let read_file = file.clone();
    let read = move || WRITER.is_idle().then(|| load::read(&read_file));
    let reload = move |loaded: Loaded, cx: &mut App| apply(&file, loaded, cx);
    let task = reload_on_change(&folder, RELOAD_DELAY, is_settings_change, read, reload, cx);
    cx.global_mut::<Settings>().watcher = task;
}

fn apply(file: &Path, loaded: Loaded, cx: &mut App) {
    let Some(loaded) = loaded.filter(|_| WRITER.is_idle()) else {
        return;
    };
    match loaded {
        Ok(values) => replace_all(values, cx),
        Err(error) => log::warn!("ignoring changes to {}: {error}", file.display()),
    }
}
