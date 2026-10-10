use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

use gpui_kit::{App, AppContext as _};
use serde::Serialize;

use super::write_json_atomic;

pub(crate) struct QueuedWriter {
    latest: AtomicU64,
    finished: AtomicU64,
    writing: Mutex<()>,
}

impl QueuedWriter {
    pub(crate) const fn new() -> Self {
        Self {
            latest: AtomicU64::new(0),
            finished: AtomicU64::new(0),
            writing: Mutex::new(()),
        }
    }

    pub(crate) fn save<T>(&'static self, file: PathBuf, value: T, cx: &App)
    where
        T: Serialize + Send + 'static,
    {
        let generation = self.latest.fetch_add(1, Ordering::SeqCst) + 1;
        cx.background_spawn(async move {
            let _writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
            if self.latest.load(Ordering::SeqCst) == generation
                && let Err(error) = write_json_atomic(&file, &value)
            {
                log::warn!("failed to save {}: {error:#}", file.display());
            }
            self.finished.fetch_max(generation, Ordering::SeqCst);
        })
        .detach();
    }

    pub(crate) fn save_now<T: Serialize + ?Sized>(&self, file: &Path, value: &T) {
        let generation = self.latest.fetch_add(1, Ordering::SeqCst) + 1;
        let _writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);
        if let Err(error) = write_json_atomic(file, value) {
            log::warn!("failed to save {}: {error:#}", file.display());
        }
        self.finished.fetch_max(generation, Ordering::SeqCst);
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.finished.load(Ordering::SeqCst) == self.latest.load(Ordering::SeqCst)
    }
}
