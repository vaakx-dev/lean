use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use notify::ErrorKind;

#[derive(Clone)]
pub(super) struct Health(Arc<AtomicBool>);

impl Default for Health {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl Health {
    pub(super) fn is_ok(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub(super) fn fail(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub(super) fn report(&self, error: &notify::Error) {
        if is_harmless(error) {
            log::debug!("file watcher: {error}");
        } else {
            log::warn!("file watcher error: {error}");
            self.fail();
        }
    }
}

fn is_harmless(error: &notify::Error) -> bool {
    match &error.kind {
        ErrorKind::PathNotFound | ErrorKind::WatchNotFound => true,
        ErrorKind::Io(error) => matches!(
            error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
        ),
        _ => false,
    }
}
