use gpui_kit::*;

use super::ImageViewer;
use super::zoom::{ZOOM_STEP, Zoom, clamp_scale, displayed_length, rescaled_offset, wheel_factor};

pub(super) struct Drag {
    start: Point<Pixels>,
    offset: Point<Pixels>,
}

impl ImageViewer {
    pub(super) fn current_scale(&self) -> Option<f32> {
        let image = self.image_size()?;
        self.zoom.scale(image, self.scroll_handle.bounds().size)
    }

    pub(super) fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.zoom_by(ZOOM_STEP, None, cx);
    }

    pub(super) fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.zoom_by(1. / ZOOM_STEP, None, cx);
    }

    pub(super) fn zoom_to_fit(&mut self, cx: &mut Context<Self>) {
        self.zoom = Zoom::Fit;
        self.scroll_handle.set_offset(Point::default());
        cx.notify();
    }

    pub(super) fn zoom_to_actual_size(&mut self, cx: &mut Context<Self>) {
        self.set_scale(1., None, cx);
    }

    pub(super) fn zoom_with_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(window.line_height());
        self.zoom_by(wheel_factor(delta.y), Some(event.position), cx);
    }

    fn zoom_by(&mut self, factor: f32, anchor: Option<Point<Pixels>>, cx: &mut Context<Self>) {
        if let Some(scale) = self.current_scale() {
            self.set_scale(scale * factor, anchor, cx);
        }
    }

    fn set_scale(&mut self, scale: f32, anchor: Option<Point<Pixels>>, cx: &mut Context<Self>) {
        let (Some(image), Some(old_scale)) = (self.image_size(), self.current_scale()) else {
            return;
        };
        let new_scale = clamp_scale(scale);
        let viewport = self.scroll_handle.bounds();
        let anchor = anchor
            .map(|anchor| anchor - viewport.origin)
            .unwrap_or_else(|| point(viewport.size.width / 2., viewport.size.height / 2.));
        let offset = self.scroll_handle.offset();
        let x = rescaled_offset(
            viewport.size.width.as_f32(),
            anchor.x.as_f32(),
            offset.x.as_f32(),
            displayed_length(image.width, old_scale).as_f32(),
            displayed_length(image.width, new_scale).as_f32(),
        );
        let y = rescaled_offset(
            viewport.size.height.as_f32(),
            anchor.y.as_f32(),
            offset.y.as_f32(),
            displayed_length(image.height, old_scale).as_f32(),
            displayed_length(image.height, new_scale).as_f32(),
        );
        self.scroll_handle.set_offset(point(px(x), px(y)));
        self.zoom = Zoom::Scale(new_scale);
        cx.notify();
    }

    pub(super) fn can_pan(&self) -> bool {
        let max = self.scroll_handle.max_offset();
        max.x > Pixels::ZERO || max.y > Pixels::ZERO
    }

    pub(super) fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub(super) fn start_drag(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        if self.can_pan() {
            self.drag = Some(Drag {
                start: event.position,
                offset: self.scroll_handle.offset(),
            });
            cx.notify();
        }
    }

    pub(super) fn drag_to(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(drag) = &self.drag else {
            return;
        };
        if event.pressed_button != Some(MouseButton::Left) {
            self.end_drag(cx);
            return;
        }
        let max = self.scroll_handle.max_offset();
        let offset = drag.offset + (event.position - drag.start);
        self.scroll_handle.set_offset(point(
            offset.x.clamp(-max.x, Pixels::ZERO),
            offset.y.clamp(-max.y, Pixels::ZERO),
        ));
        cx.notify();
    }

    pub(super) fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.drag.take().is_some() {
            cx.notify();
        }
    }
}
