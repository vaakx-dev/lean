use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::{App, AppContext as _, SharedString, Task};

use super::IgnoreCache;

pub(crate) struct DirectoryEntry {
    pub(crate) path: PathBuf,
    pub(crate) name: SharedString,
    pub(crate) is_dir: bool,
}

pub(super) fn read_directory(
    path: &Path,
    ignore: IgnoreCache,
    cx: &App,
) -> Task<Result<Vec<DirectoryEntry>>> {
    let path = path.to_path_buf();
    cx.background_spawn(async move { list_directory(&path, &ignore) })
}

fn list_directory(path: &Path, ignore: &IgnoreCache) -> Result<Vec<DirectoryEntry>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        let entry_path = entry.path();
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
    ignore.with_rules(path, |rules| {
        if let Some(rules) = rules {
            entries.retain(|entry| !rules.is_ignored(&entry.path, entry.is_dir));
        }
    });
    entries.sort_by_cached_key(|entry| (!entry.is_dir, entry.name.to_lowercase()));
    Ok(entries)
}
