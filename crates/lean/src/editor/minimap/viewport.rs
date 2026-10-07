use gpui_kit::component::input::{EditorState, RopeExt as _};
use gpui_kit::*;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct EditorViewport {
    pub(super) top_line: f32,
    pub(super) visible_lines: f32,
    pub(super) line_count: usize,
    pub(super) line_height: Pixels,
    pub(super) scroll_offset: Point<Pixels>,
    hidden_lines_above: f32,
}

impl EditorViewport {
    pub(super) fn read(editor: &EditorState) -> Option<Self> {
        let line_height = editor.line_height().filter(|height| *height > px(0.))?;
        let first_visible_line = editor.visible_row_range()?.start;
        let scroll_offset = editor.scroll_offset();
        let top_row = (-scroll_offset.y).max(px(0.)) / line_height;
        let hidden_lines_above = (first_visible_line as f32 - top_row.floor()).max(0.);
        Some(Self {
            top_line: top_row + hidden_lines_above,
            visible_lines: editor.input_bounds().size.height / line_height,
            line_count: editor.text().lines_len(),
            line_height,
            scroll_offset,
            hidden_lines_above,
        })
    }

    pub(super) fn clamp_top_line(&self, top_line: f32) -> f32 {
        top_line.clamp(0., (self.line_count as f32 - 1.).max(0.))
    }

    pub(super) fn scroll_offset_at(&self, top_line: f32) -> Point<Pixels> {
        let row = (top_line - self.hidden_lines_above).max(0.);
        point(self.scroll_offset.x, -(self.line_height * row))
    }
}
