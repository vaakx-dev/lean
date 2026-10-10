use std::collections::HashSet;
use std::path::{Path, PathBuf};

use notify::event::ModifyKind;
use notify::{Event, EventKind};

const MARKERS: &[&str] = &["pyvenv.cfg", "conda-meta"];

pub(super) fn is_relevant(event: &Event, known: &HashSet<PathBuf>) -> bool {
    match event.kind {
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_)) => {
            event
                .paths
                .iter()
                .any(|path| is_marker(path) || known.contains(path) || path.is_dir())
        }
        EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) => false,
        _ => event.paths.iter().any(|path| is_marker(path)),
    }
}

fn is_marker(path: &Path) -> bool {
    path.components().any(|component| {
        MARKERS
            .iter()
            .any(|marker| component.as_os_str() == *marker)
    })
}
