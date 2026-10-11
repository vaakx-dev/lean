use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gpui_kit::{App, Global};
use serde::{Deserialize, Serialize};

use crate::storage::{QueuedWriter, read_state, state_file};

const FILE_NAME: &str = "sessions.json";

static WRITER: QueuedWriter = QueuedWriter::new();

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(default)]
pub(super) struct Session {
    pub(super) files: Vec<PathBuf>,
    pub(super) active: Option<PathBuf>,
}

struct Sessions(BTreeMap<PathBuf, Session>);

impl Global for Sessions {}

pub(super) fn session(root: &Path, cx: &mut App) -> Option<Session> {
    loaded(cx).0.get(root).cloned()
}

pub(super) fn record(root: &Path, session: Session, cx: &mut App) {
    if loaded(cx).0.get(root) == Some(&session) {
        return;
    }
    let sessions = &mut cx.global_mut::<Sessions>().0;
    if session.files.is_empty() {
        sessions.remove(root);
    } else {
        sessions.insert(root.to_path_buf(), session);
    }
    let snapshot = sessions.clone();
    if let Some(file) = state_file(FILE_NAME) {
        WRITER.save(file, snapshot, cx);
    }
}

fn loaded(cx: &mut App) -> &Sessions {
    if !cx.has_global::<Sessions>() {
        let sessions = read_state(FILE_NAME).unwrap_or_default();
        cx.set_global(Sessions(sessions));
    }
    cx.global::<Sessions>()
}
