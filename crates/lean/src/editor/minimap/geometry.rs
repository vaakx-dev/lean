use std::ops::Range;

use gpui_kit::*;

use super::viewport::EditorViewport;

const COLUMN_WIDTH: f32 = 1.;
const TEXT_INSET: f32 = 6.;
const MIN_SLIDER_HEIGHT: f32 = 12.;

#[derive(Clone, Copy)]
pub(super) struct Geometry {
    bounds: Bounds<Pixels>,
    line_height: f32,
    line_count: usize,
    scroll_top: f32,
    slider_top: f32,
    slider_height: f32,
    lines_per_pixel: f32,
}

pub(super) fn columns_for(width: Pixels) -> usize {
    ((f32::from(width) - TEXT_INSET) / COLUMN_WIDTH).max(0.) as usize
}

impl Geometry {
    pub(super) fn new(bounds: Bounds<Pixels>, viewport: &EditorViewport, line_height: f32) -> Self {
        let height = f32::from(bounds.size.height);
        let line_count = viewport.line_count as f32;
        let slider_height = (viewport.visible_lines * line_height)
            .max(MIN_SLIDER_HEIGHT)
            .min(height);
        let slider_travel = height - slider_height;
        let scrollable_lines = (line_count - viewport.visible_lines).max(0.);
        let overflow = (line_count * line_height - height).max(0.);
        let progress = if scrollable_lines > 0. {
            (viewport.top_line / scrollable_lines).clamp(0., 1.)
        } else {
            0.
        };
        let scroll_top = progress * overflow;
        let slider_top = if overflow > 0. {
            progress * slider_travel
        } else {
            (viewport.top_line * line_height).min(slider_travel).max(0.)
        };
        let lines_per_pixel = if overflow > 0. && slider_travel > 0. {
            scrollable_lines / slider_travel
        } else {
            1. / line_height
        };
        Self {
            bounds,
            line_height,
            line_count: viewport.line_count,
            scroll_top,
            slider_top,
            slider_height,
            lines_per_pixel,
        }
    }

    pub(super) fn with_height(height: Pixels, viewport: &EditorViewport, line_height: f32) -> Self {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(0.), height));
        Self::new(bounds, viewport, line_height)
    }

    pub(super) fn visible_lines(&self) -> Range<usize> {
        let height = f32::from(self.bounds.size.height);
        let first = (self.scroll_top / self.line_height).floor() as usize;
        let last = ((self.scroll_top + height) / self.line_height).ceil() as usize;
        let end = last.min(self.line_count);
        first.min(end)..end
    }

    pub(super) fn block_bounds(&self, line: usize, column: u8, width: u8) -> Bounds<Pixels> {
        let top = line as f32 * self.line_height - self.scroll_top;
        let left = TEXT_INSET + f32::from(column) * COLUMN_WIDTH;
        Bounds::new(
            self.bounds.origin + point(px(left), px(top)),
            size(px(f32::from(width) * COLUMN_WIDTH), px(self.line_height)),
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
        (f32::from(y - self.bounds.top()) + self.scroll_top) / self.line_height
    }

    pub(super) fn lines_per_pixel(&self) -> f32 {
        self.lines_per_pixel
    }
}
