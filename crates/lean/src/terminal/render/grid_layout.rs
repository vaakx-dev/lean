mod scan;
mod text_batch;

use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::Term;
use gpui_kit::{App, PaintQuad, Pixels, Point, ShapedLine, TextAlign, Window};

use super::super::session::TerminalSession;
use super::appearance::TerminalAppearance;
use super::cursor::CursorCell;
use super::metrics::CellMetrics;
use scan::GridScan;

pub(crate) struct GridLayout {
    backgrounds: Vec<PaintQuad>,
    lines: Vec<(Point<Pixels>, ShapedLine)>,
    cursor: Option<PaintQuad>,
    line_height: Pixels,
}

impl GridLayout {
    pub(crate) fn build(
        session: &TerminalSession,
        appearance: &TerminalAppearance,
        metrics: &CellMetrics,
        origin: Point<Pixels>,
        window: &Window,
    ) -> Self {
        let (scan, cursor) = Self::scan(&*session.lock(), appearance);
        Self {
            backgrounds: scan.background_quads(origin, metrics),
            lines: scan.shape_lines(origin, appearance, metrics, window),
            cursor: cursor
                .and_then(|cursor| cursor.quad(origin, metrics, appearance.palette.cursor())),
            line_height: metrics.line_height,
        }
    }

    fn scan<T: EventListener>(
        term: &Term<T>,
        appearance: &TerminalAppearance,
    ) -> (GridScan, Option<CursorCell>) {
        let content = term.renderable_content();
        let cursor = CursorCell::visible(
            &content.cursor,
            content.display_offset,
            term.screen_lines(),
            appearance.cursor,
        );
        let mut scan = GridScan::new(appearance.palette, *content.colors);
        for indexed in content.display_iter {
            scan.push(indexed, content.display_offset, cursor.as_ref());
        }
        (scan, cursor)
    }

    pub(crate) fn paint(self, window: &mut Window, cx: &mut App) {
        for quad in self.backgrounds {
            window.paint_quad(quad);
        }
        for (origin, line) in &self.lines {
            if let Err(error) =
                line.paint(*origin, self.line_height, TextAlign::Left, None, window, cx)
            {
                log::debug!("failed to paint a terminal line: {error:#}");
            }
        }
        if let Some(cursor) = self.cursor {
            window.paint_quad(cursor);
        }
    }
}
