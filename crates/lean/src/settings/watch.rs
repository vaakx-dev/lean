use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui_kit::App;
use notify::Event;

use super::global::{Settings, replace_all};
use super::{WRITER, load};
use crate::watcher::{FolderWatch, is_noise};

const RELOAD_DELAY: Duration = Duration::from_millis(150);

pub(super) fn start(file: PathBuf, cx: &mut App) {
    let Some(folder) = file.parent().map(Path::to_path_buf) else {
        return;
    };
    if let Err(error) = std::fs::create_dir_all(&folder) {
        log::warn!("failed to create {}: {error}", folder.display());
        return;
    }
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
    let mut changes = match FolderWatch::new(RELOAD_DELAY, is_settings_change) {
        Ok(changes) => changes,
        Err(error) => {
            log::warn!("failed to watch {}: {error}", file.display());
            return;
        }
    };
    if let Err(error) = changes.watch(&folder) {
        log::warn!("failed to watch {}: {error}", folder.display());
        return;
    }
    let task = cx.spawn(async move |cx| {
        while changes.next_change(cx).await {
            cx.update(|cx| reload(&file, cx));
        }
    });
    cx.global_mut::<Settings>().watcher = Some(task);
}

fn reload(file: &Path, cx: &mut App) {
    if !WRITER.is_idle() {
        return;
    }
    match load::read(file) {
        Ok(values) => replace_all(values, cx),
        Err(error) => log::warn!("ignoring changes to {}: {error}", file.display()),
    }
}
