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
            let cell_size = CELL.as_f32() * window.scale_factor();
            window.paint_quad(fill(area, base));
            window.paint_quad(fill(area, gpui_kit::checkerboard(cell, cell_size)));
        },
    )
    .absolute()
    .size_full()
}
