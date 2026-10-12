use std::fs;
use std::path::{Path, PathBuf};

use gpui_kit::component::Rope;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::storage::state_file;

const BACKUP_EXTENSION: &str = "json";

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
    folder: PathBuf,
}

impl BackupStore {
    pub(super) fn for_project(root: &Path) -> Option<Self> {
        let name = format!("unsaved/{:016x}", path_hash(root));
        Some(Self {
            folder: state_file(&name)?,
        })
    }

    pub(super) fn file_for(&self, path: &Path) -> PathBuf {
        self.folder
            .join(format!("{:016x}.{BACKUP_EXTENSION}", path_hash(path)))
    }

    pub(super) fn stored_files(&self) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(&self.folder) else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|file| file.extension().is_some_and(|ext| ext == BACKUP_EXTENSION))
            .collect()
    }
}

pub(super) fn read_backup(file: &Path) -> Option<BackupFile> {
    let bytes = fs::read(file)
        .inspect_err(|error| log::warn!("failed to read {}: {error}", file.display()))
        .ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|error| log::warn!("failed to parse {}: {error}", file.display()))
        .ok()
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
