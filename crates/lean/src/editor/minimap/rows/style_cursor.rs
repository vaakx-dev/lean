use std::ops::Range;

use gpui_kit::*;

pub(super) struct StyleCursor<'a> {
    styles: &'a [(Range<usize>, HighlightStyle)],
    index: usize,
    foreground: Hsla,
}

impl<'a> StyleCursor<'a> {
    pub(super) fn new(styles: &'a [(Range<usize>, HighlightStyle)], foreground: Hsla) -> Self {
        Self {
            styles,
            index: 0,
            foreground,
        }
    }

    pub(super) fn color_at(&mut self, offset: usize) -> Hsla {
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
