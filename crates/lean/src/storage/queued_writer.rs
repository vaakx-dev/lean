use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, Once, PoisonError, TryLockError};

use gpui_kit::{App, AppContext as _};
use serde::Serialize;

use super::write_json_atomic;

type PendingWrite = Box<dyn FnOnce(&Path) -> io::Result<()> + Send>;

pub(crate) struct QueuedWriter {
    pending: Mutex<BTreeMap<PathBuf, PendingWrite>>,
    writing: Mutex<()>,
    flush_on_quit: Once,
}

impl QueuedWriter {
    pub(crate) const fn new() -> Self {
        Self {
            pending: Mutex::new(BTreeMap::new()),
            writing: Mutex::new(()),
            flush_on_quit: Once::new(),
        }
    }

    pub(crate) fn save<T>(&'static self, file: PathBuf, value: T, cx: &App)
    where
        T: Serialize + Send + 'static,
    {
        self.flush_on_quit.call_once(|| {
            cx.on_app_quit(move |_| {
                self.flush();
                async {}
            })
            .detach();
        });
        let write: PendingWrite = Box::new(move |file: &Path| write_json_atomic(file, &value));
        lock(&self.pending).insert(file.clone(), write);
        cx.background_spawn(async move {
            let _writing = lock(&self.writing);
            let write = lock(&self.pending).remove(&file);
            if let Some(write) = write {
                log_failure(&file, write(&file));
            }
        })
        .detach();
    }

    pub(crate) fn save_now<T: Serialize + ?Sized>(&self, file: &Path, value: &T) {
        let _writing = lock(&self.writing);
        lock(&self.pending).remove(file);
        log_failure(file, write_json_atomic(file, value));
    }

    pub(crate) fn flush(&self) {
        let _writing = lock(&self.writing);
        let pending = std::mem::take(&mut *lock(&self.pending));
        for (file, write) in pending {
            log_failure(&file, write(&file));
        }
    }

    pub(crate) fn is_idle(&self) -> bool {
        let writing = self.writing.try_lock();
        !matches!(writing, Err(TryLockError::WouldBlock)) && lock(&self.pending).is_empty()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn log_failure(file: &Path, written: io::Result<()>) {
    if let Err(error) = written {
        log::warn!("failed to save {}: {error}", file.display());
    }
}
