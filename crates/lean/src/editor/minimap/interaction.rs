use gpui_kit::*;

use super::Minimap;
use super::geometry::Geometry;

#[derive(Clone, Copy)]
pub(super) struct Drag {
    pointer_y: Pixels,
    top_line: f32,
    lines_per_pixel: f32,
}

pub(super) fn register_mouse_listeners(
    minimap: &Entity<Minimap>,
    geometry: Geometry,
    hitbox: &Hitbox,
    window: &mut Window,
) {
    window.on_mouse_event({
        let minimap = minimap.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if phase.bubble() && event.button == MouseButton::Left && hitbox.is_hovered(window) {
                minimap.update(cx, |minimap, cx| {
                    minimap.start_drag(event.position.y, &geometry, cx)
                });
                cx.stop_propagation();
            }
        }
    });
    window.on_mouse_event({
        let minimap = minimap.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseMoveEvent, phase, window, cx| {
            if phase.bubble() {
                let hovered = hitbox.is_hovered(window);
                let pressed = event.pressed_button == Some(MouseButton::Left);
                minimap.update(cx, |minimap, cx| {
                    minimap.track_pointer(event.position.y, hovered, pressed, cx)
                });
            }
        }
    });
    window.on_mouse_event({
        let minimap = minimap.clone();
        move |event: &MouseUpEvent, phase, _, cx| {
            if phase.bubble() && event.button == MouseButton::Left {
                minimap.update(cx, |minimap, cx| minimap.end_drag(cx));
            }
        }
    });
    window.on_mouse_event({
        let minimap = minimap.clone();
        let hitbox = hitbox.clone();
        move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase.bubble() && hitbox.should_handle_scroll(window) {
                minimap.update(cx, |minimap, cx| minimap.scroll_by(&event.delta, cx));
                cx.stop_propagation();
            }
        }
    });
}

impl Minimap {
    fn start_drag(&mut self, pointer_y: Pixels, geometry: &Geometry, cx: &mut Context<Self>) {
        let Some(viewport) = self.viewport else {
            return;
        };
        let top_line = if geometry.is_on_slider(pointer_y) {
            viewport.top_line
        } else {
            let centred = geometry.line_at(pointer_y) - viewport.visible_lines / 2.;
            let top_line = viewport.clamp_top_line(centred);
            self.scroll_to(top_line, cx);
            top_line
        };
        self.drag = Some(Drag {
            pointer_y,
            top_line,
            lines_per_pixel: geometry.lines_per_pixel(),
        });
        cx.notify();
    }

    fn track_pointer(
        &mut self,
        pointer_y: Pixels,
        hovered: bool,
        pressed: bool,
        cx: &mut Context<Self>,
    ) {
        if hovered != self.hovered {
            self.hovered = hovered;
            cx.notify();
        }
        let Some(drag) = self.drag else {
            return;
        };
        if pressed {
            let moved = f32::from(pointer_y - drag.pointer_y);
            self.scroll_to(drag.top_line + moved * drag.lines_per_pixel, cx);
        } else {
            self.end_drag(cx);
        }
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.drag.take().is_some() {
            cx.notify();
        }
    }

    fn scroll_to(&mut self, top_line: f32, cx: &mut Context<Self>) {
        let Some(viewport) = self.viewport else {
            return;
        };
        let offset = viewport.scroll_offset_at(viewport.clamp_top_line(top_line));
        self.editor
            .update(cx, |editor, cx| editor.set_scroll_offset(offset, cx));
    }

    fn scroll_by(&mut self, delta: &ScrollDelta, cx: &mut Context<Self>) {
        let Some(viewport) = self.viewport else {
            return;
        };
        let step = delta.pixel_delta(viewport.line_height);
        let offset = viewport.scroll_offset + point(px(0.), step.y);
        self.editor
            .update(cx, |editor, cx| editor.set_scroll_offset(offset, cx));
    }
}
