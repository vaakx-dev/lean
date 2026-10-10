mod appearance;
mod cell_style;
mod cursor;
mod grid_layout;
mod metrics;
mod palette;

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::{IntoElement, Pixels, Point, Styled, canvas};

use super::session::TerminalSession;
use grid_layout::GridLayout;
use metrics::CellMetrics;

pub(crate) use appearance::{CursorAppearance, TerminalAppearance};
pub(crate) use palette::{Palette, rgb_from_hsla};

pub(crate) fn terminal_canvas(
    session: Rc<TerminalSession>,
    appearance: TerminalAppearance,
    grid_origin: Rc<Cell<Point<Pixels>>>,
) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            grid_origin.set(bounds.origin);
            let metrics = CellMetrics::measure(&appearance, window);
            session.resize(metrics.fit(bounds.size));
            GridLayout::build(&session, &appearance, &metrics, bounds.origin, window)
        },
        |_, layout, window, cx| layout.paint(window, cx),
    )
    .flex_1()
    .min_h_0()
    .w_full()
}
