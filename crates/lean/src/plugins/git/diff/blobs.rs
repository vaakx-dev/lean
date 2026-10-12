use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context as _, Result};

use super::base::DiffBase;
use crate::editor::file_text::decode_text;
use crate::plugins::git::repo::{RepoFile, head_commit};

const GITLINK_MODE: u32 = 0o160000;
const BINARY_PROBE_LENGTH: usize = 8000;

pub(super) enum Content {
    Missing,
    Binary,
    Text(String),
}

impl Content {
    pub(super) fn decode(bytes: Vec<u8>) -> Self {
        let probe = &bytes[..bytes.len().min(BINARY_PROBE_LENGTH)];
        if probe.contains(&0) {
            return Content::Binary;
        }
        match decode_text(bytes) {
            Ok((text, _)) => Content::Text(text),
            Err(_) => Content::Binary,
        }
    }

    pub(super) fn into_text(self) -> Option<String> {
        match self {
            Content::Missing => Some(String::new()),
            Content::Binary => None,
            Content::Text(text) => Some(text),
        }
    }
}

pub(super) fn read_base(file: &RepoFile, base: DiffBase) -> Result<Content> {
    let repository = &file.repository;
    let blob = match base {
        DiffBase::Head => head_blob(repository, &file.relative_path)?,
        DiffBase::Index => index_blob(repository, &repository.index()?, &file.relative_path)?,
    };
    Ok(blob_content(blob))
}

pub(super) fn read_working(path: &Path) -> Result<Content> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Content::decode(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Content::Missing),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

pub(super) fn blob_content(blob: Option<git2::Blob<'_>>) -> Content {
    match blob {
        Some(blob) if blob.is_binary() => Content::Binary,
        Some(blob) => Content::decode(blob.content().to_vec()),
        None => Content::Missing,
    }
}

pub(super) fn index_entry(index: &git2::Index, path: &Path) -> Option<git2::IndexEntry> {
    index
        .get_path(path, 0)
        .filter(|entry| entry.mode != GITLINK_MODE)
}

pub(super) fn index_blob<'repo>(
    repository: &'repo git2::Repository,
    index: &git2::Index,
    path: &Path,
) -> Result<Option<git2::Blob<'repo>>> {
    match index_entry(index, path) {
        Some(entry) => Ok(Some(repository.find_blob(entry.id)?)),
        None => Ok(None),
    }
}

pub(super) fn head_blob<'repo>(
    repository: &'repo git2::Repository,
    path: &Path,
) -> Result<Option<git2::Blob<'repo>>> {
    let Some(head) = head_commit(repository)? else {
        return Ok(None);
    };
    let entry = match head.tree()?.get_path(path) {
        Ok(entry) => entry,
        Err(error) if error.code() == git2::ErrorCode::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if entry.kind() != Some(git2::ObjectType::Blob) {
        return Ok(None);
    }
    Ok(Some(repository.find_blob(entry.id())?))
}
