use std::rc::Rc;

use gpui_kit::*;

use super::Minimap;
use super::geometry::Geometry;
use super::interaction::register_mouse_listeners;
use super::rows::Rows;
use super::viewport::EditorViewport;

const TEXT_OPACITY: f32 = 0.6;

pub(super) struct MinimapElement {
    minimap: Entity<Minimap>,
    rows: Vec<Rc<Rows>>,
    viewport: Option<EditorViewport>,
    height: Pixels,
    slider_color: Hsla,
}

impl MinimapElement {
    pub(super) fn new(
        minimap: Entity<Minimap>,
        rows: Vec<Rc<Rows>>,
        viewport: Option<EditorViewport>,
        height: Pixels,
        slider_color: Hsla,
    ) -> Self {
        Self {
            minimap,
            rows,
            viewport,
            height,
            slider_color,
        }
    }

    fn paint_lines(&self, geometry: &Geometry, window: &mut Window) {
        let visible = geometry.visible_lines();
        for rows in &self.rows {
            let lines = rows.lines();
            for line in visible.start.max(lines.start)..visible.end.min(lines.end) {
                for block in rows.line(line) {
                    window.paint_quad(fill(
                        geometry.block_bounds(line, block.column, block.width),
                        rows.color(block).opacity(TEXT_OPACITY),
                    ));
                }
            }
        }
    }

    fn report_height(&self, height: Pixels, cx: &mut App) {
        if height == self.height {
            return;
        }
        let minimap = self.minimap.clone();
        cx.defer(move |cx| minimap.update(cx, |minimap, cx| minimap.set_height(height, cx)));
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
        self.report_height(bounds.size.height, cx);
        let Some(viewport) = self.viewport else {
            return;
        };
        let geometry = Geometry::new(bounds, &viewport);
        window.set_cursor_style(CursorStyle::Arrow, hitbox);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            self.paint_lines(&geometry, window);
            window.paint_quad(fill(geometry.slider_bounds(), self.slider_color));
        });
        register_mouse_listeners(&self.minimap, geometry, hitbox, window);
    }
}
