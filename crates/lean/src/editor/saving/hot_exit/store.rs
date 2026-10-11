use std::fs;
use std::path::{Path, PathBuf};

use gpui_kit::App;
use gpui_kit::component::Rope;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::storage::{QueuedWriter, state_file};

static WRITER: QueuedWriter = QueuedWriter::new();

#[derive(Serialize, Deserialize)]
pub(super) struct Backup {
    root: PathBuf,
    pub(super) files: Vec<BackupFile>,
}

#[derive(Serialize, Deserialize)]
pub(in crate::editor) struct BackupFile {
    pub(super) path: PathBuf,
    pub(super) base_hash: u64,
    #[serde(
        serialize_with = "serialize_rope",
        deserialize_with = "deserialize_rope"
    )]
    pub(super) text: Rope,
}

#[derive(Clone)]
pub(super) struct BackupStore {
    root: PathBuf,
    file: PathBuf,
}

impl BackupStore {
    pub(super) fn for_project(root: &Path) -> Option<Self> {
        let name = format!("unsaved/{:016x}.json", path_hash(root));
        Some(Self {
            root: root.to_path_buf(),
            file: state_file(&name)?,
        })
    }

    pub(super) fn exists(&self) -> bool {
        self.file.exists()
    }

    pub(super) fn read(&self) -> Option<Backup> {
        let bytes = match fs::read(&self.file) {
            Ok(bytes) => bytes,
            Err(error) => {
                log::debug!(
                    "no unsaved files to restore from {}: {error}",
                    self.file.display()
                );
                return None;
            }
        };
        match serde_json::from_slice::<Backup>(&bytes) {
            Ok(backup) if backup.root == self.root => Some(backup),
            Ok(_) => None,
            Err(error) => {
                log::warn!("failed to read {}: {error}", self.file.display());
                None
            }
        }
    }

    pub(super) fn save(&self, files: Vec<BackupFile>, cx: &App) {
        WRITER.save(self.file.clone(), self.backup(files), cx);
    }

    pub(super) fn save_now(&self, files: Vec<BackupFile>) {
        WRITER.save_now(&self.file, &self.backup(files));
    }

    fn backup(&self, files: Vec<BackupFile>) -> Backup {
        Backup {
            root: self.root.clone(),
            files,
        }
    }
}

fn path_hash(path: &Path) -> u64 {
    path.as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn serialize_rope<S: Serializer>(text: &Rope, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(text)
}

fn deserialize_rope<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Rope, D::Error> {
    String::deserialize(deserializer).map(|text| Rope::from_str(&text))
}
