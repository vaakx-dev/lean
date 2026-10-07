use alacritty_terminal::term::{RenderableCursor, point_to_viewport};
use alacritty_terminal::vte::ansi::CursorShape;
use gpui_kit::{
    BorderStyle, Bounds, Hsla, PaintQuad, Pixels, Point, fill, outline, point, px, size,
};

use super::appearance::CursorAppearance;
use super::metrics::CellMetrics;

const CURSOR_THICKNESS: Pixels = px(2.);

pub(crate) struct CursorCell {
    line: usize,
    column: usize,
    shape: CursorShape,
}

impl CursorCell {
    pub(crate) fn visible(
        cursor: &RenderableCursor,
        display_offset: usize,
        screen_lines: usize,
        appearance: CursorAppearance,
    ) -> Option<Self> {
        if appearance == CursorAppearance::Hidden || cursor.shape == CursorShape::Hidden {
            return None;
        }
        let position = point_to_viewport(display_offset, cursor.point)?;
        if position.line >= screen_lines {
            return None;
        }
        let shape = match appearance {
            CursorAppearance::Hollow => CursorShape::HollowBlock,
            CursorAppearance::Solid | CursorAppearance::Hidden => cursor.shape,
        };
        Some(Self {
            line: position.line,
            column: position.column.0,
            shape,
        })
    }

    pub(crate) fn is_solid_block_at(&self, line: usize, column: usize) -> bool {
        self.shape == CursorShape::Block && self.line == line && self.column == column
    }

    pub(crate) fn quad(
        &self,
        origin: Point<Pixels>,
        metrics: &CellMetrics,
        color: Hsla,
    ) -> Option<PaintQuad> {
        let cell_origin = metrics.cell_origin(origin, self.line, self.column);
        let cell = Bounds::new(cell_origin, size(metrics.cell_width, metrics.line_height));
        match self.shape {
            CursorShape::Beam => Some(fill(
                Bounds::new(cell_origin, size(CURSOR_THICKNESS, metrics.line_height)),
                color,
            )),
            CursorShape::Underline => Some(fill(
                Bounds::new(
                    point(
                        cell_origin.x,
                        cell_origin.y + metrics.line_height - CURSOR_THICKNESS,
                    ),
                    size(metrics.cell_width, CURSOR_THICKNESS),
                ),
                color,
            )),
            CursorShape::HollowBlock => Some(outline(cell, color, BorderStyle::Solid)),
            CursorShape::Block | CursorShape::Hidden => None,
        }
    }
}
