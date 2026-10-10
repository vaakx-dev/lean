use gpui_kit::component::input::EditorState;
use gpui_kit::*;

use super::super::rows::{Rows, RowsJob};
use super::super::style::MinimapStyle;
use super::chunk_lines;

pub(in super::super) struct ChunkJob {
    version: u64,
    indices: Vec<usize>,
    rows: RowsJob,
}

pub(in super::super) struct BuiltChunks {
    pub(super) version: u64,
    pub(super) chunks: Vec<(usize, Rows)>,
}

impl ChunkJob {
    pub(in super::super) fn new(
        editor: &EditorState,
        indices: Vec<usize>,
        version: u64,
        foreground: Hsla,
        style: &MinimapStyle,
    ) -> Self {
        let ranges = indices.iter().map(|&index| chunk_lines(index)).collect();
        Self {
            version,
            indices,
            rows: RowsJob::new(editor, ranges, foreground, style),
        }
    }

    pub(in super::super) fn run(self) -> BuiltChunks {
        BuiltChunks {
            version: self.version,
            chunks: self.indices.into_iter().zip(self.rows.run()).collect(),
        }
    }
}
