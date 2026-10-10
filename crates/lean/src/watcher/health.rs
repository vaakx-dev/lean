use std::io;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use notify::ErrorKind;

const TOO_MANY_OPEN_FILES: i32 = 24;

#[derive(Clone)]
pub(super) enum WatchFailure {
    WatchLimit,
    InstanceLimit,
    Other(String),
}

impl WatchFailure {
    pub(super) fn of(error: &notify::Error) -> Self {
        match &error.kind {
            ErrorKind::MaxFilesWatch => Self::WatchLimit,
            ErrorKind::Io(error) if error.raw_os_error() == Some(TOO_MANY_OPEN_FILES) => {
                Self::InstanceLimit
            }
            _ => Self::Other(error.to_string()),
        }
    }

    pub(super) fn message(&self) -> String {
        match self {
            Self::WatchLimit => with_hint(
                "Lean reached the system limit on watched folders, so changes made outside Lean may not show up.",
                "fs.inotify.max_user_watches=524288",
            ),
            Self::InstanceLimit => with_hint(
                "Lean couldn't start a file watcher because too many are already open, so changes made outside Lean may not show up.",
                "fs.inotify.max_user_instances=512",
            ),
            Self::Other(error) => format!(
                "File watching stopped ({error}), so changes made outside Lean may not show up. Lean will keep retrying."
            ),
        }
    }
}

fn with_hint(problem: &str, sysctl: &str) -> String {
    if cfg!(target_os = "linux") {
        format!("{problem} Raise the limit with \"sudo sysctl {sysctl}\"; Lean will keep retrying.")
    } else {
        format!("{problem} Lean will keep retrying.")
    }
}

#[derive(Clone, Default)]
pub(super) struct Health(Arc<Mutex<Option<WatchFailure>>>);

impl Health {
    pub(super) fn is_ok(&self) -> bool {
        self.state().is_none()
    }

    pub(super) fn failure(&self) -> Option<WatchFailure> {
        self.state().clone()
    }

    pub(super) fn restore(&self) {
        *self.state() = None;
    }

    pub(super) fn fail(&self, failure: WatchFailure) {
        let mut state = self.state();
        if state.is_none() {
            *state = Some(failure);
        }
    }

    pub(super) fn report(&self, error: &notify::Error) {
        if is_harmless(error) {
            log::debug!("file watcher: {error}");
        } else {
            log::warn!("file watcher error: {error}");
            self.fail(WatchFailure::of(error));
        }
    }

    fn state(&self) -> MutexGuard<'_, Option<WatchFailure>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
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
