use std::sync::Arc;

use gpui_kit::component::scroll::{ScrollableElement as _, ScrollbarAxis};
use gpui_kit::component::{ActiveTheme as _, Icon, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::keymap::{KEY_CONTEXT, bind_actions};
use super::zoom::{PADDING, Zoom, displayed_length};
use super::{Decoded, ImageViewer};
use crate::plugins::media::picture::{Fit, picture};
use crate::plugins::media::settings::{IMAGE_TOOLBAR, WHEEL, WheelAction};

impl ImageViewer {
    fn render_stage(&self, image: &Arc<RenderImage>, cx: &Context<Self>) -> Stateful<Div> {
        let pannable = self.can_pan();
        let dragging = self.is_dragging();
        let fit = if self.upscale_small {
            Fit::Contain
        } else {
            Fit::ScaleDown
        };
        let content = match self.zoom {
            Zoom::Fit => div()
                .size_full()
                .p(PADDING)
                .child(picture("image", image, fit, cx).size_full()),
            Zoom::Scale(scale) => {
                let size = image.size(0);
                let width = displayed_length(size.width, scale);
                let height = displayed_length(size.height, scale);
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .w(width + PADDING * 2.)
                    .h(height + PADDING * 2.)
                    .min_w_full()
                    .min_h_full()
                    .child(
                        picture("image", image, Fit::Contain, cx)
                            .flex_none()
                            .w(width)
                            .h(height),
                    )
            }
        };
        let content = content
            .id("image-content")
            .when(pannable && !dragging, |content| content.cursor_grab())
            .when(dragging, |content| content.cursor_grabbing())
            .on_scroll_wheel(cx.listener(|viewer, event: &ScrollWheelEvent, window, cx| {
                if wheel_zooms(WHEEL.get(cx), event.modifiers.secondary()) {
                    cx.stop_propagation();
                    viewer.zoom_with_wheel(event, window, cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|viewer, event: &MouseDownEvent, _, cx| viewer.start_drag(event, cx)),
            )
            .on_mouse_move(
                cx.listener(|viewer, event: &MouseMoveEvent, _, cx| viewer.drag_to(event, cx)),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|viewer, _: &MouseUpEvent, _, cx| viewer.end_drag(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|viewer, _: &MouseUpEvent, _, cx| viewer.end_drag(cx)),
            );
        div()
            .id("image-scroll")
            .size_full()
            .overflow_scroll()
            .track_scroll(&self.scroll_handle)
            .child(content)
    }
}

impl Render for ImageViewer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx: &Context<Self> = cx;
        let theme = cx.theme();
        let root = div()
            .id("image-viewer")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(theme.background);
        let root = bind_actions(root, cx);
        match &self.decoded {
            Decoded::Loading => root,
            Decoded::Failed(error) => root.child(render_message(
                "icons/file-image.svg",
                format!("This image can't be displayed: {error}").into(),
                cx,
            )),
            Decoded::Ready(image) => root
                .child(self.render_stage(image, cx))
                .scrollbar(&self.scroll_handle, ScrollbarAxis::Both)
                .when(IMAGE_TOOLBAR.get(cx), |root| {
                    root.child(self.render_toolbar(cx))
                }),
        }
    }
}

fn wheel_zooms(action: WheelAction, secondary: bool) -> bool {
    match action {
        WheelAction::ZoomWithCtrl => secondary,
        WheelAction::Zoom => true,
        WheelAction::Pan => false,
    }
}

fn render_message(icon: &'static str, message: SharedString, cx: &App) -> Div {
    let theme = cx.theme();
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_3()
        .p_4()
        .child(
            Icon::empty()
                .path(icon)
                .size(px(48.))
                .text_color(theme.muted_foreground),
        )
        .child(
            div()
                .max_w(px(480.))
                .text_sm()
                .text_center()
                .text_color(theme.muted_foreground)
                .child(message),
        )
}
