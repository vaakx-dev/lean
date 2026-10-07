use std::ops::Range;

use gpui_kit::component::input::{Rope, RopeExt as _};
use gpui_kit::*;

const MAX_COLUMNS: usize = 120;
const TAB_COLUMNS: usize = 4;

#[derive(Clone, Copy)]
pub(super) struct Block {
    pub(super) column: u16,
    pub(super) width: u16,
    pub(super) color: Hsla,
}

#[derive(Default)]
pub(super) struct Document {
    pub(super) lines: Vec<Vec<Block>>,
}

impl Document {
    pub(super) fn build(
        text: &Rope,
        styles: &[(Range<usize>, HighlightStyle)],
        foreground: Hsla,
    ) -> Self {
        let mut colors = StyleCursor {
            styles,
            index: 0,
            foreground,
        };
        let mut line_start = 0;
        let lines = text
            .iter_lines()
            .map(|line| {
                let blocks = line_blocks(line.chars(), line_start, &mut colors);
                line_start += line.len() + 1;
                blocks
            })
            .collect();
        Self { lines }
    }
}

fn line_blocks(
    characters: impl Iterator<Item = char>,
    line_start: usize,
    colors: &mut StyleCursor,
) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut column = 0;
    let mut offset = line_start;
    for character in characters {
        if column >= MAX_COLUMNS {
            break;
        }
        if character == '\t' {
            column += TAB_COLUMNS - column % TAB_COLUMNS;
        } else if character.is_whitespace() {
            column += 1;
        } else {
            let color = colors.color_at(offset);
            match blocks.last_mut() {
                Some(block)
                    if block.color == color
                        && usize::from(block.column + block.width) == column =>
                {
                    block.width += 1
                }
                _ => blocks.push(Block {
                    column: column as u16,
                    width: 1,
                    color,
                }),
            }
            column += 1;
        }
        offset += character.len_utf8();
    }
    blocks
}

struct StyleCursor<'a> {
    styles: &'a [(Range<usize>, HighlightStyle)],
    index: usize,
    foreground: Hsla,
}

impl StyleCursor<'_> {
    fn color_at(&mut self, offset: usize) -> Hsla {
        while self
            .styles
            .get(self.index)
            .is_some_and(|(range, _)| range.end <= offset)
        {
            self.index += 1;
        }
        match self.styles.get(self.index) {
            Some((range, style)) if range.start <= offset => style.color.unwrap_or(self.foreground),
            _ => self.foreground,
        }
    }
}
