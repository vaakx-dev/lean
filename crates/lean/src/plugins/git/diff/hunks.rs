use std::ops::Range;
use std::path::Path;

use anyhow::Result;
use gpui_kit::component::Theme;

use super::base::DiffBase;
use super::blobs::{Content, is_in_index, read_base};
use crate::plugins::GutterMarker;
use crate::plugins::git::repo::RepoFile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HunkKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Hunk {
    pub(crate) kind: HunkKind,
    pub(crate) new_lines: Range<usize>,
}

pub(crate) fn hunks_for(
    repo_root: &Path,
    path: &Path,
    text: &str,
    base: DiffBase,
) -> Result<Vec<Hunk>> {
    let file = RepoFile::open(repo_root, path)?;
    if file.repository.is_path_ignored(&file.relative_path)? {
        return Ok(Vec::new());
    }
    match read_base(&file, base)? {
        Content::Text(old) => line_hunks(&old, text),
        Content::Missing if !is_in_index(&file)? => Ok(Vec::new()),
        Content::Missing => line_hunks("", text),
        Content::Binary => Ok(Vec::new()),
    }
}

pub(crate) fn gutter_markers(hunks: &[Hunk], theme: &Theme) -> Vec<GutterMarker> {
    hunks
        .iter()
        .map(|hunk| GutterMarker {
            lines: hunk.new_lines.clone(),
            color: match hunk.kind {
                HunkKind::Added => theme.success,
                HunkKind::Modified => theme.info,
                HunkKind::Deleted => theme.danger,
            },
        })
        .collect()
}

fn line_hunks(old: &str, new: &str) -> Result<Vec<Hunk>> {
    let mut options = diff_options(0);
    let patch = git2::Patch::from_buffers(
        old.as_bytes(),
        None,
        new.as_bytes(),
        None,
        Some(&mut options),
    )?;
    (0..patch.num_hunks())
        .map(|index| read_hunk(&patch, index))
        .collect()
}

pub(super) fn diff_options(context_lines: u32) -> git2::DiffOptions {
    let mut options = git2::DiffOptions::new();
    options
        .context_lines(context_lines)
        .interhunk_lines(0)
        .indent_heuristic(true)
        .force_text(true);
    options
}

pub(super) fn line_text(content: &[u8]) -> String {
    String::from_utf8_lossy(content)
        .trim_end_matches(['\n', '\r'])
        .to_string()
}

fn read_hunk(patch: &git2::Patch<'_>, index: usize) -> Result<Hunk> {
    let (hunk, _) = patch.hunk(index)?;
    let new_lines = line_range(hunk.new_start(), hunk.new_lines());
    let kind = if hunk.old_lines() == 0 {
        HunkKind::Added
    } else if new_lines.is_empty() {
        HunkKind::Deleted
    } else {
        HunkKind::Modified
    };
    Ok(Hunk { kind, new_lines })
}

fn line_range(start: u32, count: u32) -> Range<usize> {
    let start = start as usize;
    let count = count as usize;
    if count == 0 {
        start..start
    } else {
        start - 1..start - 1 + count
    }
}
