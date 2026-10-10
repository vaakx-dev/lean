use gpui_kit::*;

use super::activity_bar::ACTIVITY_BAR_WIDTH;
use super::settings::SIDEBAR_WIDTH_RANGE;
use super::{GAP, Workspace};

const MIN_EDITOR_WIDTH: Pixels = px(320.);

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

    pub(super) fn visible_sidebar_width(&self, window: &Window) -> Pixels {
        self.sidebar_width.min(max_sidebar_width(window))
    }

    fn stop_sidebar_resize(&mut self, cx: &mut Context<Self>) {
        self.resizing_sidebar = false;
        cx.notify();
    }
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

pub(super) fn track_sidebar_resize(
    element: Stateful<Div>,
    cx: &Context<Workspace>,
) -> Stateful<Div> {
    element
        .cursor_col_resize()
        .on_mouse_move(
            cx.listener(|workspace, event: &MouseMoveEvent, window, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    workspace.stop_sidebar_resize(cx);
                    return;
                }
                let (min, _) = SIDEBAR_WIDTH_RANGE;
                let width = event.position.x - ACTIVITY_BAR_WIDTH - GAP;
                workspace.sidebar_width = width.min(max_sidebar_width(window)).max(px(min));
                cx.notify();
            }),
        )
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
