use std::path::{Path, PathBuf};
use std::time::Instant;

use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
use notify::{Event, EventKind};

use super::metadata::{MetadataPath, SharedMetadata, is_dot_git};
use super::shared_paths::SharedPaths;
use super::skipped::SkippedFolders;

pub(super) struct RawEvent {
    pub(super) paths: Vec<PathBuf>,
    pub(super) rename: Option<(PathBuf, PathBuf)>,
    pub(super) structural: bool,
    pub(super) metadata: bool,
    pub(super) rescan: bool,
    pub(super) retry: bool,
    pub(super) received: Instant,
}

impl RawEvent {
    pub(super) fn rescan() -> Self {
        Self {
            metadata: true,
            rescan: true,
            ..Self::empty()
        }
    }

    pub(super) fn retry() -> Self {
        Self {
            retry: true,
            ..Self::empty()
        }
    }

    pub(super) fn empty() -> Self {
        Self {
            paths: Vec::new(),
            rename: None,
            structural: false,
            metadata: false,
            rescan: false,
            retry: false,
            received: Instant::now(),
        }
    }
}

#[derive(Clone)]
pub(super) struct EventFilter {
    pub(super) metadata: SharedMetadata,
    pub(super) skipped: SkippedFolders,
    pub(super) open_files: SharedPaths,
}

impl EventFilter {
    pub(super) fn filter(&self, event: Event) -> Option<RawEvent> {
        if event.need_rescan() {
            return Some(RawEvent::rescan());
        }
        if is_noise(event.kind) {
            return None;
        }
        let is_rename = matches!(
            event.kind,
            EventKind::Modify(ModifyKind::Name(RenameMode::Both))
        ) && event.paths.len() == 2;
        let mut raw = RawEvent::empty();
        raw.structural = is_structural(event.kind);
        for path in event.paths {
            if adds_or_removes_repository(event.kind, &path) {
                raw.metadata = true;
                raw.rescan |= self.is_followed(&path);
                continue;
            }
            match self.metadata.classify(&path) {
                MetadataPath::Worktree if self.is_followed(&path) => raw.paths.push(path),
                MetadataPath::Relevant => raw.metadata = true,
                MetadataPath::IgnoreRules => raw.rescan = true,
                MetadataPath::Worktree | MetadataPath::Internal => {}
            }
        }
        if is_rename && let [from, to] = raw.paths.as_slice() {
            raw.rename = Some((from.clone(), to.clone()));
        }
        (raw.metadata || raw.rescan || !raw.paths.is_empty()).then_some(raw)
    }

    fn is_followed(&self, path: &Path) -> bool {
        !self.skipped.contains_parent_of(path) || self.open_files.contains(path)
    }
}

fn is_noise(kind: EventKind) -> bool {
    matches!(kind, EventKind::Access(access) if access != AccessKind::Close(AccessMode::Write))
}

fn is_structural(kind: EventKind) -> bool {
    !matches!(
        kind,
        EventKind::Access(_)
            | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Metadata(_) | ModifyKind::Any)
    )
}

fn adds_or_removes_repository(kind: EventKind, path: &Path) -> bool {
    is_dot_git(path)
        && matches!(
            kind,
            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(_))
        )
}
