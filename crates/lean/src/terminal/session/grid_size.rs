use alacritty_terminal::event::WindowSize;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{MIN_COLUMNS, MIN_SCREEN_LINES};
use gpui_kit::{Pixels, Size, px};

const DEFAULT_COLUMNS: usize = 80;
const DEFAULT_LINES: usize = 24;
const DEFAULT_CELL_WIDTH: Pixels = px(8.);
const DEFAULT_LINE_HEIGHT: Pixels = px(18.);
const MIN_CELL_SIZE: Pixels = px(1.);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridSize {
    pub(crate) columns: usize,
    pub(crate) lines: usize,
    pub(crate) cell_width: Pixels,
    pub(crate) line_height: Pixels,
}

impl GridSize {
    pub(crate) fn fit(available: Size<Pixels>, cell_width: Pixels, line_height: Pixels) -> Self {
        let cell_width = cell_width.max(MIN_CELL_SIZE);
        let line_height = line_height.max(MIN_CELL_SIZE);
        Self {
            columns: cells_within(available.width, cell_width).max(MIN_COLUMNS),
            lines: cells_within(available.height, line_height).max(MIN_SCREEN_LINES),
            cell_width,
            line_height,
        }
    }

    pub(crate) fn window_size(&self) -> WindowSize {
        WindowSize {
            num_lines: saturating_u16(self.lines),
            num_cols: saturating_u16(self.columns),
            cell_width: self.cell_width.as_f32() as u16,
            cell_height: self.line_height.as_f32() as u16,
        }
    }
}

impl Default for GridSize {
    fn default() -> Self {
        Self {
            columns: DEFAULT_COLUMNS,
            lines: DEFAULT_LINES,
            cell_width: DEFAULT_CELL_WIDTH,
            line_height: DEFAULT_LINE_HEIGHT,
        }
    }
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

fn cells_within(length: Pixels, cell: Pixels) -> usize {
    let count = (length / cell).floor().max(0.) as usize;
    count.min(usize::from(u16::MAX))
}

fn saturating_u16(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
