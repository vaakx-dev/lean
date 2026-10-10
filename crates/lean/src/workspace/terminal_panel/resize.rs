use std::rc::Rc;
use std::time::Duration;

use gpui_kit::base::resize_handle;
use gpui_kit::component::resize_handle_appearance;
use gpui_kit::*;

pub(super) const SAVE_DELAY: Duration = Duration::from_millis(300);

const MIN_HEIGHT: Pixels = px(120.);
const MAX_HEIGHT: Pixels = px(800.);
const MIN_EDITOR_HEIGHT: Pixels = px(160.);

pub(super) struct ResizeDrag;

pub(super) type TerminalResize = Rc<ResizeDrag>;

impl Render for ResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub(super) fn terminal_resize_handle() -> impl IntoElement {
    resize_handle::<ResizeDrag, ResizeDrag>("terminal-resize-handle", Axis::Vertical)
        .with_appearance(resize_handle_appearance())
        .on_drag(ResizeDrag, |_, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| ResizeDrag)
        })
}

pub(super) fn panel_height(requested: Pixels, viewport_height: Pixels) -> Pixels {
    let max = MAX_HEIGHT
        .min(viewport_height - MIN_EDITOR_HEIGHT)
        .max(MIN_HEIGHT);
    requested.clamp(MIN_HEIGHT, max)
}
