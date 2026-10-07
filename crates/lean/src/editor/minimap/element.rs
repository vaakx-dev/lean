use std::rc::Rc;

use gpui_kit::*;

use super::Minimap;
use super::document::Document;
use super::geometry::Geometry;
use super::interaction::register_mouse_listeners;
use super::viewport::EditorViewport;

const TEXT_OPACITY: f32 = 0.6;

pub(super) struct MinimapElement {
    minimap: Entity<Minimap>,
    document: Rc<Document>,
    viewport: Option<EditorViewport>,
    slider_color: Hsla,
}

impl MinimapElement {
    pub(super) fn new(
        minimap: Entity<Minimap>,
        document: Rc<Document>,
        viewport: Option<EditorViewport>,
        slider_color: Hsla,
    ) -> Self {
        Self {
            minimap,
            document,
            viewport,
            slider_color,
        }
    }

    fn paint_lines(&self, geometry: &Geometry, window: &mut Window) {
        let range = geometry.visible_lines(self.document.lines.len());
        let Some(lines) = self.document.lines.get(range.clone()) else {
            return;
        };
        for (line, blocks) in range.zip(lines) {
            for block in blocks {
                window.paint_quad(fill(
                    geometry.block_bounds(line, block.column, block.width),
                    block.color.opacity(TEXT_OPACITY),
                ));
            }
        }
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
        _: &mut App,
    ) {
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
