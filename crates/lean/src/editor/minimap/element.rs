use std::rc::Rc;

use gpui_kit::*;

use super::Minimap;
use super::geometry::Geometry;
use super::interaction::register_mouse_listeners;
use super::rows::Rows;
use super::style::MinimapStyle;
use super::viewport::EditorViewport;

pub(super) struct MinimapElement {
    pub(super) minimap: Entity<Minimap>,
    pub(super) rows: Vec<Rc<Rows>>,
    pub(super) viewport: Option<EditorViewport>,
    pub(super) height: Pixels,
    pub(super) style: MinimapStyle,
    pub(super) slider_color: Option<Hsla>,
    pub(super) refresh_pending: bool,
}

impl MinimapElement {
    fn paint_lines(&self, geometry: &Geometry, window: &mut Window) {
        let visible = geometry.visible_lines();
        for rows in &self.rows {
            let lines = rows.lines();
            for line in visible.start.max(lines.start)..visible.end.min(lines.end) {
                for block in rows.line(line) {
                    window.paint_quad(fill(
                        geometry.block_bounds(line, block.column, block.width),
                        rows.color(block).opacity(self.style.text_opacity),
                    ));
                }
            }
        }
    }

    fn report_painted(&self, height: Pixels, cx: &mut App) {
        if height == self.height && !self.refresh_pending {
            return;
        }
        let minimap = self.minimap.clone();
        cx.defer(move |cx| minimap.update(cx, |minimap, cx| minimap.painted(height, cx)));
    }
}

impl IntoElement for MinimapElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for MinimapElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.report_painted(bounds.size.height, cx);
        let Some(viewport) = self.viewport else {
            return;
        };
        let geometry = Geometry::new(bounds, &viewport, self.style.line_height);
        window.set_cursor_style(CursorStyle::Arrow, hitbox);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            self.paint_lines(&geometry, window);
            if let Some(color) = self.slider_color {
                window.paint_quad(fill(geometry.slider_bounds(), color));
            }
        });
        register_mouse_listeners(&self.minimap, geometry, hitbox, window);
    }
}
