use gpui_kit::*;

use super::activity_bar::ACTIVITY_BAR_WIDTH;
use super::{GAP, Workspace};

const MIN_SIDEBAR_WIDTH: Pixels = px(170.);
const MAX_SIDEBAR_WIDTH: Pixels = px(600.);

impl Workspace {
    pub(super) fn render_resize_handle(&self, cx: &Context<Self>) -> Stateful<Div> {
        div()
            .id("sidebar-resize-handle")
            .w(GAP)
            .h_full()
            .flex_shrink_0()
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|workspace, _, _, cx| {
                    workspace.resizing_sidebar = true;
                    cx.notify();
                }),
            )
    }

    fn stop_sidebar_resize(&mut self, cx: &mut Context<Self>) {
        self.resizing_sidebar = false;
        cx.notify();
    }
}

pub(super) fn track_sidebar_resize(
    element: Stateful<Div>,
    cx: &Context<Workspace>,
) -> Stateful<Div> {
    element
        .cursor_col_resize()
        .on_mouse_move(cx.listener(|workspace, event: &MouseMoveEvent, _, cx| {
            if event.pressed_button != Some(MouseButton::Left) {
                workspace.stop_sidebar_resize(cx);
                return;
            }
            let width = event.position.x - ACTIVITY_BAR_WIDTH - GAP;
            workspace.sidebar_width = width.clamp(MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH);
            cx.notify();
        }))
        .capture_any_mouse_up(cx.listener(|workspace, event: &MouseUpEvent, _, cx| {
            if event.button == MouseButton::Left {
                workspace.stop_sidebar_resize(cx);
            }
        }))
        .on_mouse_up_out(
            MouseButton::Left,
            cx.listener(|workspace, _, _, cx| workspace.stop_sidebar_resize(cx)),
        )
}
