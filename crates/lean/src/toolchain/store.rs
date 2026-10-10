use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, PoisonError};

use gpui_kit::App;

use crate::storage::{QueuedWriter, state_file};

const FILE_NAME: &str = "toolchains.json";

pub(super) type Choices = BTreeMap<String, Option<PathBuf>>;
type SavedChoices = BTreeMap<PathBuf, Choices>;

static SAVED: Mutex<Option<SavedChoices>> = Mutex::new(None);
static WRITER: QueuedWriter = QueuedWriter::new();
static FLUSH_ON_QUIT: Once = Once::new();

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
    FLUSH_ON_QUIT.call_once(|| {
        cx.on_app_quit(|_| {
            flush();
            async {}
        })
        .detach();
    });
    WRITER.save(file, snapshot, cx);
}

fn flush() {
    if WRITER.is_idle() {
        return;
    }
    if let Some(file) = state_file(FILE_NAME) {
        with_saved(|saved| WRITER.save_now(&file, &*saved));
    }
}

fn with_saved<R>(use_saved: impl FnOnce(&mut SavedChoices) -> R) -> R {
    let mut saved = SAVED.lock().unwrap_or_else(PoisonError::into_inner);
    use_saved(saved.get_or_insert_with(read_file))
}

fn read_file() -> SavedChoices {
    let Some(file) = state_file(FILE_NAME) else {
        return SavedChoices::new();
    };
    let contents = match std::fs::read_to_string(&file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return SavedChoices::new(),
        Err(error) => {
            log::warn!("failed to read {}: {error}", file.display());
            return SavedChoices::new();
        }
    };
    serde_json::from_str(&contents).unwrap_or_else(|error| {
        log::warn!("failed to parse {}: {error}", file.display());
        SavedChoices::new()
    })
}
