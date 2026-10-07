use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    App, Bounds, ContentMask, Hsla, IntoElement, PathBuilder, Pixels, Point, Styled, Window,
    canvas, fill, point, px, size,
};

use super::graph::GraphRow;

const LANE_WIDTH: Pixels = px(12.);
const MAX_VISIBLE_LANES: usize = 8;
const LINE_WIDTH: Pixels = px(1.5);
const DOT_RADIUS: Pixels = px(3.5);

pub(super) fn graph_column(row: GraphRow, lanes: usize) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                paint_graph(&row, bounds, window, cx)
            })
        },
    )
    .w(LANE_WIDTH * lanes.clamp(1, MAX_VISIBLE_LANES) as f32)
    .h_full()
    .flex_shrink_0()
}

fn paint_graph(row: &GraphRow, bounds: Bounds<Pixels>, window: &mut Window, cx: &App) {
    let top = bounds.top();
    let middle = bounds.center().y;
    let bottom = bounds.bottom();
    let lane_x = |lane: usize| bounds.left() + LANE_WIDTH * (lane as f32 + 0.5);
    for lane in row.passing_lanes() {
        paint_line(
            point(lane_x(lane), top),
            point(lane_x(lane), bottom),
            lane_color(lane, cx),
            window,
        );
    }
    let dot = point(lane_x(row.lane), middle);
    if row.has_line_above(row.lane) {
        paint_line(point(dot.x, top), dot, lane_color(row.lane, cx), window);
    }
    if row.has_line_below(row.lane) {
        paint_line(dot, point(dot.x, bottom), lane_color(row.lane, cx), window);
    }
    for &lane in &row.merging_from {
        paint_line(point(lane_x(lane), top), dot, lane_color(lane, cx), window);
    }
    for &lane in &row.branching_to {
        paint_line(
            dot,
            point(lane_x(lane), bottom),
            lane_color(lane, cx),
            window,
        );
    }
    paint_dot(dot, row, window, cx);
}

fn paint_line(from: Point<Pixels>, to: Point<Pixels>, color: Hsla, window: &mut Window) {
    let mut builder = PathBuilder::stroke(LINE_WIDTH);
    builder.move_to(from);
    if from.x == to.x {
        builder.line_to(to);
    } else {
        let bend = from.y + (to.y - from.y) * 0.5;
        builder.cubic_bezier_to(to, point(from.x, bend), point(to.x, bend));
    }
    match builder.build() {
        Ok(path) => window.paint_path(path, color),
        Err(error) => log::debug!("failed to build a commit graph line: {error:?}"),
    }
}

fn paint_dot(center: Point<Pixels>, row: &GraphRow, window: &mut Window, cx: &App) {
    let color = lane_color(row.lane, cx);
    let background = if row.is_merge {
        cx.theme().sidebar
    } else {
        color
    };
    let diameter = DOT_RADIUS * 2.;
    window.paint_quad(
        fill(
            Bounds::centered_at(center, size(diameter, diameter)),
            background,
        )
        .corner_radii(DOT_RADIUS)
        .border_widths(LINE_WIDTH)
        .border_color(color),
    );
}

fn lane_color(lane: usize, cx: &App) -> Hsla {
    let theme = cx.theme();
    let colors = [
        theme.primary,
        theme.blue,
        theme.green,
        theme.yellow,
        theme.cyan,
        theme.magenta,
        theme.red,
    ];
    colors[lane % colors.len()]
}
