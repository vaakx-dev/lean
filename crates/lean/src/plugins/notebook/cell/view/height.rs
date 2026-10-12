use std::cell::Cell as Memo;

use gpui_kit::component::{Rope, RopeExt as _};
use gpui_kit::*;

use crate::plugins::notebook::cell::Cell;
use crate::plugins::notebook::cell_options::CellFont;
use crate::plugins::notebook::settings::{LINE_NUMBERS, SOFT_WRAP};

pub(super) const EDITOR_PADDING: Pixels = px(8.);
const EDITOR_SLACK: Pixels = px(2.);
const NUMBER_MARGIN: Pixels = px(6.);
const WRAP_MARGIN: Pixels = px(10.);
const MIN_NUMBER_DIGITS: usize = 3;
const MAX_NUMBER_DIGITS: usize = 7;

#[derive(Clone, Copy)]
struct WrappedRows {
    revision: u64,
    width: Pixels,
    rows: usize,
}

#[derive(Default)]
pub(in crate::plugins::notebook) struct RowCache(Memo<Option<WrappedRows>>);

impl RowCache {
    pub(in crate::plugins::notebook) fn clear(&self) {
        self.0.set(None);
    }

    fn get_or_compute(
        &self,
        revision: u64,
        width: Pixels,
        compute: impl FnOnce() -> usize,
    ) -> usize {
        if let Some(cached) = self.0.get()
            && cached.revision == revision
            && cached.width == width
        {
            return cached.rows;
        }
        let rows = compute();
        self.0.set(Some(WrappedRows {
            revision,
            width,
            rows,
        }));
        rows
    }
}

pub(super) fn editor_height(cell: &Cell, style: &CellFont, cx: &App) -> Pixels {
    let rows = row_count(cell, style, cx);
    style.line_pixels() * rows as f32 + EDITOR_PADDING * 2. + EDITOR_SLACK
}

fn row_count(cell: &Cell, style: &CellFont, cx: &App) -> usize {
    let editor = cell.editor.read(cx);
    let text = editor.text();
    let lines = text.lines_len().max(1);
    match editor.text_bounds() {
        Some(bounds) if SOFT_WRAP.get(cx) => {
            cell.rows
                .get_or_compute(cell.revision, bounds.size.width, || {
                    let width = bounds.size.width - gutter_width(lines, style, cx) - WRAP_MARGIN;
                    wrapped_rows(text, width, style, cx).max(lines)
                })
        }
        _ => lines,
    }
}

fn gutter_width(lines: usize, style: &CellFont, cx: &App) -> Pixels {
    if !LINE_NUMBERS.get(cx) {
        return NUMBER_MARGIN;
    }
    let digits = (lines.ilog10() as usize + 1).clamp(MIN_NUMBER_DIGITS, MAX_NUMBER_DIGITS);
    let text_system = cx.text_system();
    let font_id = text_system.resolve_font(&font(style.family(cx)));
    let advance = text_system
        .ch_advance(font_id, style.size)
        .unwrap_or(style.size);
    advance * digits as f32 + NUMBER_MARGIN
}

fn wrapped_rows(text: &Rope, width: Pixels, style: &CellFont, cx: &App) -> usize {
    if width <= Pixels::ZERO {
        return text.lines_len();
    }
    let mut wrapper = cx
        .text_system()
        .line_wrapper(font(style.family(cx)), style.size);
    String::from(text)
        .split('\n')
        .map(|line| {
            let fragments = [LineFragment::text(line)];
            1 + wrapper
                .wrap_line(&fragments, width, IndentAdjustment::SameIndent)
                .count()
        })
        .sum()
}
