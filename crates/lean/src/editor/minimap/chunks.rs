mod build;

use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use super::rows::Rows;

pub(super) use build::{BuiltChunks, ChunkJob};

const CHUNK_LINES: usize = 1024;
const MAX_CHUNKS: usize = 16;

struct Chunk {
    version: u64,
    rows: Rc<Rows>,
}

#[derive(Default)]
pub(super) struct Chunks {
    chunks: HashMap<usize, Chunk>,
}

impl Chunks {
    pub(super) fn rows(&self) -> Vec<Rc<Rows>> {
        self.chunks
            .values()
            .map(|chunk| chunk.rows.clone())
            .collect()
    }

    pub(super) fn has_missing(&self, lines: &Range<usize>) -> bool {
        chunk_indices(lines).any(|index| !self.chunks.contains_key(&index))
    }

    pub(super) fn needs_build(&self, lines: &Range<usize>, version: u64) -> bool {
        chunk_indices(lines).any(|index| self.is_outdated(index, version))
    }

    pub(super) fn outdated(&self, lines: &Range<usize>, version: u64) -> Vec<usize> {
        chunk_indices(lines)
            .filter(|&index| self.is_outdated(index, version))
            .collect()
    }

    pub(super) fn insert(&mut self, built: BuiltChunks, visible: &Range<usize>) -> bool {
        let mut visible_changed = false;
        for (index, rows) in built.chunks {
            let rows = match self.chunks.remove(&index) {
                Some(old) if *old.rows == rows => old.rows,
                _ => {
                    visible_changed |= chunk_indices(visible).contains(&index);
                    Rc::new(rows)
                }
            };
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

    fn is_outdated(&self, index: usize, version: u64) -> bool {
        self.chunks
            .get(&index)
            .is_none_or(|chunk| chunk.version != version)
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
