use alacritty_terminal::term::cell::Cell;
use gpui_kit::{
    Pixels, Point, ShapedLine, SharedString, StrikethroughStyle, TextRun, UnderlineStyle, Window,
    px,
};

use super::super::appearance::TerminalAppearance;
use super::super::cell_style::CellStyle;
use super::super::metrics::CellMetrics;

const DECORATION_THICKNESS: Pixels = px(1.);

pub(super) struct TextBatch {
    line: usize,
    column: usize,
    cells: usize,
    wide: bool,
    text: String,
    style: CellStyle,
}

impl TextBatch {
    pub(super) fn start(line: usize, column: usize, wide: bool, style: CellStyle) -> Self {
        Self {
            line,
            column,
            cells: 0,
            wide,
            text: String::new(),
            style,
        }
    }

    pub(super) fn accepts(&self, line: usize, column: usize, style: CellStyle) -> bool {
        !self.wide && self.line == line && self.column + self.cells == column && self.style == style
    }

    pub(super) fn append(&mut self, cell: &Cell, cells: usize) {
        self.text.push(if cell.c == '\t' { ' ' } else { cell.c });
        if let Some(zerowidth) = cell.zerowidth() {
            self.text.extend(zerowidth);
        }
        self.cells += cells;
    }

    pub(super) fn shape(
        self,
        origin: Point<Pixels>,
        appearance: &TerminalAppearance,
        metrics: &CellMetrics,
        window: &Window,
    ) -> (Point<Pixels>, ShapedLine) {
        let run = self.text_run(appearance);
        let line = window.text_system().shape_line(
            SharedString::from(self.text),
            appearance.font_size,
            &[run],
            Some(metrics.cell_width),
        );
        (metrics.cell_origin(origin, self.line, self.column), line)
    }

    fn text_run(&self, appearance: &TerminalAppearance) -> TextRun {
        TextRun {
            len: self.text.len(),
            font: self.style.font(&appearance.font),
            color: self.style.foreground,
            background_color: None,
            underline: self.style.underline.then_some(UnderlineStyle {
                thickness: DECORATION_THICKNESS,
                color: None,
                wavy: false,
            }),
            strikethrough: self.style.strikethrough.then_some(StrikethroughStyle {
                thickness: DECORATION_THICKNESS,
                color: None,
            }),
        }
    }
}
