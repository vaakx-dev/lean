use std::ops::Range;

use anyhow::Result;

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

pub(super) fn line_hunks(old: &str, new: &str) -> Result<Vec<Hunk>> {
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
