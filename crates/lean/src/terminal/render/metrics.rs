use gpui_kit::{Pixels, Point, Size, Window, point};

use super::appearance::TerminalAppearance;
use crate::terminal::session::GridSize;

const LINE_HEIGHT_RATIO: f32 = 1.3;
const FALLBACK_WIDTH_RATIO: f32 = 0.6;

pub(crate) fn line_height(font_size: Pixels) -> Pixels {
    (font_size * LINE_HEIGHT_RATIO).round()
}

pub(crate) struct CellMetrics {
    pub(crate) cell_width: Pixels,
    pub(crate) line_height: Pixels,
}

impl CellMetrics {
    pub(crate) fn measure(appearance: &TerminalAppearance, window: &Window) -> Self {
        let text_system = window.text_system();
        let font_id = text_system.resolve_font(&appearance.font);
        let cell_width = text_system
            .em_advance(font_id, appearance.font_size)
            .unwrap_or_else(|error| {
                log::debug!("failed to measure the terminal font: {error:#}");
                appearance.font_size * FALLBACK_WIDTH_RATIO
            });
        Self {
            cell_width,
            line_height: line_height(appearance.font_size),
        }
    }

    pub(crate) fn fit(&self, available: Size<Pixels>) -> GridSize {
        GridSize::fit(available, self.cell_width, self.line_height)
    }

    pub(crate) fn cell_origin(
        &self,
        origin: Point<Pixels>,
        line: usize,
        column: usize,
    ) -> Point<Pixels> {
        point(
            origin.x + self.cell_width * column,
            origin.y + self.line_height * line,
        )
    }
}
