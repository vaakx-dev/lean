use alacritty_terminal::grid::Indexed;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::color::Colors;
use alacritty_terminal::term::point_to_viewport;
use gpui_kit::{Bounds, Hsla, PaintQuad, Pixels, Point, ShapedLine, Window, fill, size};

use super::super::appearance::TerminalAppearance;
use super::super::cell_style::CellStyle;
use super::super::cursor::CursorCell;
use super::super::metrics::CellMetrics;
use super::super::palette::Palette;
use super::text_batch::TextBatch;

struct BackgroundSpan {
    line: usize,
    column: usize,
    cells: usize,
    color: Hsla,
}

pub(super) struct GridScan<'a> {
    palette: Palette,
    overrides: &'a Colors,
    batches: Vec<TextBatch>,
    backgrounds: Vec<BackgroundSpan>,
}

impl<'a> GridScan<'a> {
    pub(super) fn new(palette: Palette, overrides: &'a Colors) -> Self {
        Self {
            palette,
            overrides,
            batches: Vec::new(),
            backgrounds: Vec::new(),
        }
    }

    pub(super) fn push(
        &mut self,
        indexed: Indexed<&Cell>,
        display_offset: usize,
        cursor: Option<&CursorCell>,
    ) {
        let cell = indexed.cell;
        if cell
            .flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
        {
            return;
        }
        let Some(position) = point_to_viewport(display_offset, indexed.point) else {
            return;
        };
        let (line, column) = (position.line, position.column.0);
        let mut style = CellStyle::of(cell, &self.palette, self.overrides);
        if cursor.is_some_and(|cursor| cursor.is_solid_block_at(line, column)) {
            style = style.under_cursor(&self.palette);
        }
        let wide = cell.flags.contains(Flags::WIDE_CHAR);
        let cells = if wide { 2 } else { 1 };
        if style.background != self.palette.background() {
            self.push_background(line, column, cells, style.background);
        }
        if !is_blank(cell) || style.is_decorated() {
            self.push_text(line, column, cells, wide, cell, style);
        }
    }

    pub(super) fn background_quads(
        &self,
        origin: Point<Pixels>,
        metrics: &CellMetrics,
    ) -> Vec<PaintQuad> {
        self.backgrounds
            .iter()
            .map(|span| {
                let bounds = Bounds::new(
                    metrics.cell_origin(origin, span.line, span.column),
                    size(metrics.cell_width * span.cells, metrics.line_height),
                );
                fill(bounds, span.color)
            })
            .collect()
    }

    pub(super) fn shape_lines(
        self,
        origin: Point<Pixels>,
        appearance: &TerminalAppearance,
        metrics: &CellMetrics,
        window: &Window,
    ) -> Vec<(Point<Pixels>, ShapedLine)> {
        self.batches
            .into_iter()
            .map(|batch| batch.shape(origin, appearance, metrics, window))
            .collect()
    }

    fn push_background(&mut self, line: usize, column: usize, cells: usize, color: Hsla) {
        if let Some(span) = self.backgrounds.last_mut()
            && span.line == line
            && span.column + span.cells == column
            && span.color == color
        {
            span.cells += cells;
            return;
        }
        self.backgrounds.push(BackgroundSpan {
            line,
            column,
            cells,
            color,
        });
    }

    fn push_text(
        &mut self,
        line: usize,
        column: usize,
        cells: usize,
        wide: bool,
        cell: &Cell,
        style: CellStyle,
    ) {
        if !wide
            && let Some(batch) = self.batches.last_mut()
            && batch.accepts(line, column, style)
        {
            batch.append(cell, cells);
            return;
        }
        let mut batch = TextBatch::start(line, column, wide, style);
        batch.append(cell, cells);
        self.batches.push(batch);
    }
}

fn is_blank(cell: &Cell) -> bool {
    let empty = matches!(cell.c, ' ' | '\t') && cell.zerowidth().is_none();
    empty || cell.flags.contains(Flags::HIDDEN)
}
