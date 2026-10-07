use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::{App, AppContext as _, SharedString, Task};

use crate::git::IgnoreRules;

pub(crate) struct DirectoryEntry {
    pub(crate) path: PathBuf,
    pub(crate) name: SharedString,
    pub(crate) is_dir: bool,
}

pub(crate) fn read_directory(path: &Path, cx: &App) -> Task<Result<Vec<DirectoryEntry>>> {
    let path = path.to_path_buf();
    cx.background_spawn(async move { list_directory(&path) })
}

fn list_directory(path: &Path) -> Result<Vec<DirectoryEntry>> {
    let ignore_rules = IgnoreRules::discover(path);
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        let entry_path = entry.path();
        if ignore_rules
            .as_ref()
            .is_some_and(|rules| rules.is_ignored(&entry_path))
        {
            continue;
        }
        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir() || (file_type.is_symlink() && entry_path.is_dir()))
            .unwrap_or_else(|error| {
                log::debug!(
                    "failed to read file type of {}: {error}",
                    entry_path.display()
                );
                false
            });
        entries.push(DirectoryEntry {
            path: entry_path,
            name: name.into(),
            is_dir,
        });
    }
    entries.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}
