use std::collections::HashSet;
use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use notify::event::ModifyKind;
use notify::{Event, EventKind};

const MARKERS: &[&str] = &["pyvenv.cfg", "conda-meta"];

struct Pending {
    known: HashSet<PathBuf>,
    touched_known: bool,
    created: Vec<PathBuf>,
}

#[derive(Clone)]
pub(super) struct Changes(Arc<Mutex<Pending>>);

impl Changes {
    pub(super) fn new(known: HashSet<PathBuf>) -> Self {
        Self(Arc::new(Mutex::new(Pending {
            known,
            touched_known: false,
            created: Vec::new(),
        })))
    }

    pub(super) fn set_known(&self, known: HashSet<PathBuf>) {
        self.lock().known = known;
    }

    pub(super) fn record(&self, event: &Event) -> bool {
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_))
        ) {
            return false;
        }
        let mut pending = self.lock();
        let mut recorded = false;
        for path in &event.paths {
            if pending.known.contains(path) {
                pending.touched_known = true;
            } else if path.is_dir() {
                pending.created.push(path.clone());
            } else {
                continue;
            }
            recorded = true;
        }
        recorded
    }

    pub(super) fn take(&self) -> (bool, Vec<PathBuf>) {
        let mut pending = self.lock();
        (
            mem::take(&mut pending.touched_known),
            mem::take(&mut pending.created),
        )
    }

    fn lock(&self) -> MutexGuard<'_, Pending> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub(super) fn has_environment(folders: &[PathBuf]) -> bool {
    folders
        .iter()
        .any(|folder| MARKERS.iter().any(|marker| folder.join(marker).exists()))
}
