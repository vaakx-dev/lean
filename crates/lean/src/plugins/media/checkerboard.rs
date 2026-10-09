use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::picture::Fit;

const CELL: Pixels = px(8.);
const CELL_OPACITY: f32 = 0.07;

pub(super) fn checkerboard(image_size: Size<DevicePixels>, fit: Fit, cx: &App) -> Canvas<()> {
    let theme = cx.theme();
    let base = theme.background;
    let cell = theme.foreground.opacity(CELL_OPACITY);
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            if image_size.width.0 <= 0 || image_size.height.0 <= 0 {
                return;
            }
            let area = fit.object_fit().get_bounds(bounds, image_size);
            paint_cells(area, base, cell, window);
        },
    )
    .absolute()
    .size_full()
}

fn paint_cells(area: Bounds<Pixels>, base: Hsla, cell: Hsla, window: &mut Window) {
    let visible = area.intersect(&window.content_mask().bounds);
    if visible.is_empty() {
        return;
    }
    window.paint_quad(fill(visible, base));
    let first_column = cell_index(visible.left() - area.left());
    let last_column = cell_index(visible.right() - area.left());
    let first_row = cell_index(visible.top() - area.top());
    let last_row = cell_index(visible.bottom() - area.top());
    for row in first_row..=last_row {
        for column in first_column..=last_column {
            if (row + column) % 2 == 0 {
                continue;
            }
            let origin = point(
                area.left() + CELL * column as f32,
                area.top() + CELL * row as f32,
            );
            let square = Bounds::new(origin, size(CELL, CELL)).intersect(&visible);
            if !square.is_empty() {
                window.paint_quad(fill(square, cell));
            }
        }
    }
}

fn cell_index(distance: Pixels) -> u32 {
    (distance / CELL).max(0.).floor() as u32
}
