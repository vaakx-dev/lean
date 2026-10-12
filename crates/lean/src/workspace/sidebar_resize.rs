use std::rc::Rc;

use gpui_kit::base::resize_handle;
use gpui_kit::component::resize_handle_appearance;
use gpui_kit::*;

use super::activity_bar::ACTIVITY_BAR_WIDTH;
use super::settings::SIDEBAR_WIDTH_RANGE;
use super::{GAP, Workspace};

const MIN_EDITOR_WIDTH: Pixels = px(320.);
const SIDEBAR_BORDER: Pixels = px(1.);

pub(super) struct SidebarDrag;

pub(super) type SidebarResize = Rc<SidebarDrag>;

impl Render for SidebarDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl Workspace {
    pub(super) fn visible_sidebar_width(&self, window: &Window) -> Pixels {
        self.sidebar_width.min(max_sidebar_width(window))
    }

    pub(super) fn resize_sidebar(
        &mut self,
        event: &DragMoveEvent<SidebarResize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (min, _) = SIDEBAR_WIDTH_RANGE;
        let requested = event.event.position.x - event.bounds.left();
        let width = requested.min(max_sidebar_width(window)).max(px(min));
        if width != self.sidebar_width {
            self.sidebar_width = width;
            cx.notify();
        }
    }
}

pub(super) fn sidebar_resize_handle() -> Div {
    div()
        .relative()
        .flex_shrink_0()
        .h_full()
        .w(GAP + SIDEBAR_BORDER)
        .ml(-SIDEBAR_BORDER)
        .child(
            resize_handle::<SidebarDrag, SidebarDrag>("sidebar-resize-handle", Axis::Horizontal)
                .with_appearance(resize_handle_appearance())
                .on_drag(SidebarDrag, |_, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| SidebarDrag)
                }),
        )
}

pub(in crate::workspace) fn clamp_sidebar_width(width: f32) -> Pixels {
    let (min, max) = SIDEBAR_WIDTH_RANGE;
    px(width.clamp(min, max))
}

fn max_sidebar_width(window: &Window) -> Pixels {
    let (min, max) = SIDEBAR_WIDTH_RANGE;
    let room = window.viewport_size().width - ACTIVITY_BAR_WIDTH - GAP * 2. - MIN_EDITOR_WIDTH;
    room.min(px(max)).max(px(min))
}
