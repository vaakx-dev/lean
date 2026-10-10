use anyhow::Result;
use gpui_kit::SharedString;

use super::super::blobs::Content;
use super::super::hunks::{diff_options, line_text};
use super::side::Side;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RowKind {
    Context,
    Added,
    Removed,
}

pub(super) enum DiffRow {
    Header(SharedString),
    Line {
        kind: RowKind,
        old_line: Option<usize>,
        new_line: Option<usize>,
    },
}

pub(super) struct DiffDocument {
    pub(super) rows: Vec<DiffRow>,
    pub(super) old: Side,
    pub(super) new: Side,
    pub(super) added: usize,
    pub(super) removed: usize,
    pub(super) notice: Option<&'static str>,
}

struct DiffRows {
    rows: Vec<DiffRow>,
    added: usize,
    removed: usize,
}

impl DiffDocument {
    pub(super) fn build(
        old: Content,
        new: Content,
        language: &str,
        context_lines: u32,
    ) -> Result<Self> {
        let (Some(old_text), Some(new_text)) = (old.into_text(), new.into_text()) else {
            return Ok(Self::with_notice("Binary files can't be shown as a diff."));
        };
        let DiffRows {
            rows,
            added,
            removed,
        } = diff_rows(&old_text, &new_text, context_lines)?;
        let notice = rows.is_empty().then_some("No changes.");
        Ok(Self {
            rows,
            old: Side::new(&old_text, language),
            new: Side::new(&new_text, language),
            added,
            removed,
            notice,
        })
    }

    fn with_notice(notice: &'static str) -> Self {
        Self {
            rows: Vec::new(),
            old: Side::empty(),
            new: Side::empty(),
            added: 0,
            removed: 0,
            notice: Some(notice),
        }
    }

    pub(super) fn number_width(&self) -> usize {
        self.old
            .line_count()
            .max(self.new.line_count())
            .to_string()
            .len()
    }
}

fn diff_rows(old: &str, new: &str, context_lines: u32) -> Result<DiffRows> {
    let mut options = diff_options(context_lines);
    let patch = git2::Patch::from_buffers(
        old.as_bytes(),
        None,
        new.as_bytes(),
        None,
        Some(&mut options),
    )?;
    let mut diff = DiffRows {
        rows: Vec::new(),
        added: 0,
        removed: 0,
    };
    for hunk_index in 0..patch.num_hunks() {
        let (hunk, line_count) = patch.hunk(hunk_index)?;
        diff.rows
            .push(DiffRow::Header(line_text(hunk.header()).into()));
        for line_index in 0..line_count {
            let line = patch.line_in_hunk(hunk_index, line_index)?;
            let kind = match line.origin_value() {
                git2::DiffLineType::Context => RowKind::Context,
                git2::DiffLineType::Addition => {
                    diff.added += 1;
                    RowKind::Added
                }
                git2::DiffLineType::Deletion => {
                    diff.removed += 1;
                    RowKind::Removed
                }
                _ => continue,
            };
            diff.rows.push(DiffRow::Line {
                kind,
                old_line: zero_based(line.old_lineno()),
                new_line: zero_based(line.new_lineno()),
            });
        }
    }
    Ok(diff)
}

fn zero_based(line: Option<u32>) -> Option<usize> {
    line.map(|line| (line as usize).saturating_sub(1))
}
