use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use gpui_kit::App;

use crate::storage::{QueuedWriter, read_state, state_file};

const FILE_NAME: &str = "toolchains.json";

pub(super) type Choices = BTreeMap<String, Option<PathBuf>>;
type SavedChoices = BTreeMap<PathBuf, Choices>;

static SAVED: Mutex<Option<SavedChoices>> = Mutex::new(None);
static WRITER: QueuedWriter = QueuedWriter::new();

pub(super) fn load(project_root: &Path) -> Choices {
    with_saved(|saved| saved.get(project_root).cloned().unwrap_or_default())
}

pub(super) fn save(project_root: &Path, language: &str, prefix: Option<PathBuf>, cx: &App) {
    let Some(file) = state_file(FILE_NAME) else {
        return;
    };
    let snapshot = with_saved(|saved| {
        saved
            .entry(project_root.to_path_buf())
            .or_default()
            .insert(language.to_owned(), prefix);
        saved.clone()
    });
    WRITER.save(file, snapshot, cx);
}

fn with_saved<R>(use_saved: impl FnOnce(&mut SavedChoices) -> R) -> R {
    let mut saved = SAVED.lock().unwrap_or_else(PoisonError::into_inner);
    use_saved(saved.get_or_insert_with(|| read_state(FILE_NAME).unwrap_or_default()))
}
