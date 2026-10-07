use std::ops::Range;

use gpui_kit::*;

use super::viewport::EditorViewport;

const LINE_HEIGHT: f32 = 2.;
const COLUMN_WIDTH: f32 = 1.;
const TEXT_INSET: f32 = 6.;

#[derive(Clone, Copy)]
pub(super) struct Geometry {
    bounds: Bounds<Pixels>,
    scroll_top: f32,
    slider_top: f32,
    slider_height: f32,
    lines_per_pixel: f32,
}

impl Geometry {
    pub(super) fn new(bounds: Bounds<Pixels>, viewport: &EditorViewport) -> Self {
        let height = f32::from(bounds.size.height);
        let line_count = viewport.line_count as f32;
        let slider_height = (viewport.visible_lines * LINE_HEIGHT).min(height);
        let slider_travel = height - slider_height;
        let scrollable_lines = (line_count - viewport.visible_lines).max(0.);
        let overflow = (line_count * LINE_HEIGHT - height).max(0.);
        let progress = if scrollable_lines > 0. {
            (viewport.top_line / scrollable_lines).clamp(0., 1.)
        } else {
            0.
        };
        let scroll_top = progress * overflow;
        let lines_per_pixel = if overflow > 0. && slider_travel > 0. {
            scrollable_lines / slider_travel
        } else {
            1. / LINE_HEIGHT
        };
        Self {
            bounds,
            scroll_top,
            slider_top: viewport.top_line * LINE_HEIGHT - scroll_top,
            slider_height,
            lines_per_pixel,
        }
    }

    pub(super) fn visible_lines(&self, line_count: usize) -> Range<usize> {
        let height = f32::from(self.bounds.size.height);
        let first = (self.scroll_top / LINE_HEIGHT).floor() as usize;
        let last = ((self.scroll_top + height) / LINE_HEIGHT).ceil() as usize;
        let end = last.min(line_count);
        first.min(end)..end
    }

    pub(super) fn block_bounds(&self, line: usize, column: u16, width: u16) -> Bounds<Pixels> {
        let top = line as f32 * LINE_HEIGHT - self.scroll_top;
        let left = TEXT_INSET + f32::from(column) * COLUMN_WIDTH;
        Bounds::new(
            self.bounds.origin + point(px(left), px(top)),
            size(px(f32::from(width) * COLUMN_WIDTH), px(LINE_HEIGHT)),
        )
    }

    pub(super) fn slider_bounds(&self) -> Bounds<Pixels> {
        Bounds::new(
            self.bounds.origin + point(px(0.), px(self.slider_top)),
            size(self.bounds.size.width, px(self.slider_height)),
        )
    }

    pub(super) fn is_on_slider(&self, y: Pixels) -> bool {
        let slider = self.slider_bounds();
        slider.top() <= y && y < slider.bottom()
    }

    pub(super) fn line_at(&self, y: Pixels) -> f32 {
        (f32::from(y - self.bounds.top()) + self.scroll_top) / LINE_HEIGHT
    }

    pub(super) fn lines_per_pixel(&self) -> f32 {
        self.lines_per_pixel
    }
}
