use std::collections::HashMap;
use std::ops::Range;

use gpui_kit::Hsla;
use gpui_kit::component::input::EditorState;

use super::rows::{Rows, RowsJob};
use super::style::MinimapStyle;

const CHUNK_LINES: usize = 1024;
const MAX_CHUNKS: usize = 16;

struct Chunk {
    version: u64,
    rows: Rows,
}

pub(super) struct ChunkJob {
    version: u64,
    indices: Vec<usize>,
    rows: RowsJob,
}

pub(super) struct BuiltChunks {
    version: u64,
    chunks: Vec<(usize, Rows)>,
}

impl ChunkJob {
    pub(super) fn new(
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

    pub(super) fn run(self) -> BuiltChunks {
        BuiltChunks {
            version: self.version,
            chunks: self.indices.into_iter().zip(self.rows.run()).collect(),
        }
    }
}

#[derive(Default)]
pub(super) struct Chunks {
    chunks: HashMap<usize, Chunk>,
}

impl Chunks {
    pub(super) fn rows(&self) -> impl Iterator<Item = &Rows> {
        self.chunks.values().map(|chunk| &chunk.rows)
    }

    pub(super) fn has_missing(&self, lines: &Range<usize>) -> bool {
        chunk_indices(lines).any(|index| !self.chunks.contains_key(&index))
    }

    pub(super) fn outdated(&self, lines: &Range<usize>, version: u64) -> Vec<usize> {
        chunk_indices(lines)
            .filter(|index| {
                self.chunks
                    .get(index)
                    .is_none_or(|chunk| chunk.version != version)
            })
            .collect()
    }

    pub(super) fn insert(&mut self, built: BuiltChunks, visible: &Range<usize>) -> bool {
        let mut visible_changed = false;
        for (index, rows) in built.chunks {
            let unchanged = self.chunks.get(&index).is_some_and(|old| old.rows == rows);
            if !unchanged {
                visible_changed |= chunk_indices(visible).contains(&index);
            }
            let version = built.version;
            self.chunks.insert(index, Chunk { version, rows });
        }
        self.evict_far_from(visible);
        visible_changed
    }

    pub(super) fn drop_past(&mut self, line_count: usize) {
        let chunk_count = line_count.div_ceil(CHUNK_LINES);
        self.chunks.retain(|&index, _| index < chunk_count);
    }

    fn evict_far_from(&mut self, visible: &Range<usize>) {
        let centre = visible.start.midpoint(visible.end) / CHUNK_LINES;
        while self.chunks.len() > MAX_CHUNKS {
            let farthest = self
                .chunks
                .keys()
                .copied()
                .max_by_key(|index| index.abs_diff(centre));
            let Some(farthest) = farthest else {
                return;
            };
            self.chunks.remove(&farthest);
        }
    }
}

fn chunk_indices(lines: &Range<usize>) -> Range<usize> {
    if lines.is_empty() {
        return 0..0;
    }
    lines.start / CHUNK_LINES..(lines.end - 1) / CHUNK_LINES + 1
}

fn chunk_lines(index: usize) -> Range<usize> {
    index * CHUNK_LINES..(index + 1) * CHUNK_LINES
}
