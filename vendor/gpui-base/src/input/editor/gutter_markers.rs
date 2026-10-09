use std::ops::Range;

use gpui::{Bounds, Context, Hsla, Pixels, Window, fill, point, px, size};

use super::EditorState;

const BAR_WIDTH: Pixels = px(3.);
const WEDGE_WIDTH: Pixels = px(5.);
const WEDGE_HALF_HEIGHT: Pixels = px(4.);

#[derive(Clone, Debug, PartialEq)]
pub struct GutterMarker {
    pub lines: Range<usize>,
    pub color: Hsla,
}

impl EditorState {
    pub fn set_gutter_markers(&mut self, markers: Vec<GutterMarker>, cx: &mut Context<Self>) {
        self.extras.gutter_markers = markers;
        cx.notify();
    }
}

pub(crate) struct GutterRow {
    pub(crate) buffer_line: usize,
    pub(crate) top: Pixels,
    pub(crate) height: Pixels,
}

pub(crate) fn paint_gutter_markers(
    markers: &[GutterMarker],
    rows: &[GutterRow],
    left: Pixels,
    window: &mut Window,
) {
    let (Some(first), Some(last)) = (rows.first(), rows.last()) else {
        return;
    };
    let visible = first.buffer_line..last.buffer_line + 1;

    for marker in markers {
        if marker.lines.is_empty() {
            paint_wedge(marker, rows, &visible, left, window);
        } else if marker.lines.start < visible.end && visible.start < marker.lines.end {
            for row in rows
                .iter()
                .filter(|row| marker.lines.contains(&row.buffer_line))
            {
                window.paint_quad(fill(
                    Bounds::new(point(left, row.top), size(BAR_WIDTH, row.height)),
                    marker.color,
                ));
            }
        }
    }
}

fn paint_wedge(
    marker: &GutterMarker,
    rows: &[GutterRow],
    visible: &Range<usize>,
    left: Pixels,
    window: &mut Window,
) {
    let line = marker.lines.start;
    if line > visible.end || line + 1 < visible.start {
        return;
    }
    let y = rows
        .iter()
        .find(|row| row.buffer_line == line)
        .map(|row| row.top)
        .or_else(|| {
            rows.iter()
                .find(|row| row.buffer_line + 1 == line)
                .map(|row| row.top + row.height)
        });
    let Some(y) = y else {
        return;
    };

    let mut builder = gpui::PathBuilder::fill();
    builder.move_to(point(left, y - WEDGE_HALF_HEIGHT));
    builder.line_to(point(left + WEDGE_WIDTH, y));
    builder.line_to(point(left, y + WEDGE_HALF_HEIGHT));
    builder.close();
    if let Ok(path) = builder.build() {
        window.paint_path(path, marker.color);
    }
}
