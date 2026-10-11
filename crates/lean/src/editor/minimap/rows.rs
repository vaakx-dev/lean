mod job;

use std::ops::Range;

use gpui_kit::*;

pub(super) use job::RowsJob;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Block {
    pub(super) column: u8,
    pub(super) width: u8,
    color: u16,
}

#[derive(Default, PartialEq)]
pub(super) struct Rows {
    first_line: usize,
    starts: Vec<u32>,
    blocks: Vec<Block>,
    palette: Vec<Hsla>,
}

impl Rows {
    pub(super) fn lines(&self) -> Range<usize> {
        self.first_line..self.first_line + self.starts.len().saturating_sub(1)
    }

    pub(super) fn line(&self, line: usize) -> &[Block] {
        let Some(row) = line.checked_sub(self.first_line) else {
            return &[];
        };
        match (self.starts.get(row), self.starts.get(row + 1)) {
            (Some(&start), Some(&end)) => &self.blocks[start as usize..end as usize],
            _ => &[],
        }
    }

    pub(super) fn color(&self, block: &Block) -> Hsla {
        self.palette[usize::from(block.color)]
    }
}

struct RowsBuilder {
    rows: Rows,
    row_start: usize,
    last_color: Option<(Hsla, u16)>,
}

impl RowsBuilder {
    fn new(first_line: usize) -> Self {
        Self {
            rows: Rows {
                first_line,
                starts: vec![0],
                ..Rows::default()
            },
            row_start: 0,
            last_color: None,
        }
    }

    fn push(&mut self, column: usize, color: Hsla) {
        let color = self.color_index(color);
        let row = &mut self.rows.blocks[self.row_start..];
        if let Some(block) = row.last_mut()
            && block.color == color
            && usize::from(block.column) + usize::from(block.width) == column
        {
            block.width += 1;
            return;
        }
        self.rows.blocks.push(Block {
            column: column as u8,
            width: 1,
            color,
        });
    }

    fn end_row(&mut self) {
        self.row_start = self.rows.blocks.len();
        self.rows.starts.push(self.row_start as u32);
    }

    fn finish(self) -> Rows {
        self.rows
    }

    fn color_index(&mut self, color: Hsla) -> u16 {
        if let Some((last, index)) = self.last_color
            && last == color
        {
            return index;
        }
        let palette = &mut self.rows.palette;
        let known = palette.iter().position(|known| *known == color);
        let index = known.unwrap_or_else(|| {
            palette.push(color);
            palette.len() - 1
        }) as u16;
        self.last_color = Some((color, index));
        index
    }
}
