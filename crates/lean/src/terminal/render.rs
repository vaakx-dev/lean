mod appearance;
mod cell_style;
mod cursor;
mod grid_layout;
mod metrics;
mod palette;

use std::rc::Rc;

use gpui_kit::{IntoElement, Styled, canvas};

use super::session::TerminalSession;
use grid_layout::GridLayout;
use metrics::CellMetrics;

pub(crate) use appearance::{CursorAppearance, TerminalAppearance};
pub(crate) use metrics::line_height;
pub(crate) use palette::{Palette, rgb_from_hsla};

pub(crate) fn terminal_canvas(
    session: Rc<TerminalSession>,
    appearance: TerminalAppearance,
) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            let metrics = CellMetrics::measure(&appearance, window);
            session.resize(metrics.fit(bounds.size));
            let term = session.lock();
            GridLayout::build(&*term, &appearance, &metrics, bounds.origin, window)
        },
        |_, layout, window, cx| layout.paint(window, cx),
    )
    .flex_1()
    .min_h_0()
    .w_full()
}
