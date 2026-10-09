mod chunks;
mod element;
mod geometry;
mod interaction;
mod refresh;
mod rows;
mod viewport;

use std::ops::Range;

use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::*;

use chunks::Chunks;
use element::MinimapElement;
use geometry::{Geometry, WIDTH};
use interaction::Drag;
use refresh::{Refresh, Styling};
use viewport::EditorViewport;

const SLIDER_OPACITY: f32 = 0.06;
const SLIDER_HOVER_OPACITY: f32 = 0.12;
const SLIDER_DRAG_OPACITY: f32 = 0.18;

pub(crate) struct Minimap {
    editor: Entity<EditorState>,
    viewport: Option<EditorViewport>,
    height: Pixels,
    chunks: Chunks,
    refresh: Refresh,
    hovered: bool,
    drag: Option<Drag>,
    _subscriptions: [Subscription; 3],
}

impl Minimap {
    pub(crate) fn new(editor: Entity<EditorState>, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.observe(&editor, |minimap, _, cx| minimap.sync_editor(cx)),
            cx.subscribe(&editor, |minimap, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    minimap.refresh.text_changed();
                    minimap.request_refresh(cx);
                }
            }),
            cx.observe_global::<Theme>(|minimap, cx| {
                minimap.refresh.restyle();
                minimap.request_refresh(cx);
                minimap.sync_editor_later(cx);
                cx.notify();
            }),
        ];
        let state = editor.read(cx);
        let viewport = EditorViewport::read(state);
        let refresh = Refresh::new(Styling::read(state));
        Self {
            editor,
            viewport,
            height: px(0.),
            chunks: Chunks::default(),
            refresh,
            hovered: false,
            drag: None,
            _subscriptions: subscriptions,
        }
    }

    fn sync_editor(&mut self, cx: &mut Context<Self>) {
        let editor = self.editor.read(cx);
        let viewport = EditorViewport::read(editor);
        let styling = Styling::read(editor);
        if viewport != self.viewport {
            self.viewport = viewport;
            if let Some(viewport) = viewport {
                self.chunks.drop_past(viewport.line_count);
            }
            cx.notify();
        }
        self.refresh.sync_styling(styling);
        self.request_refresh(cx);
    }

    fn sync_editor_later(&self, cx: &mut Context<Self>) {
        let minimap = cx.entity().downgrade();
        cx.defer(move |cx| {
            if let Err(error) = minimap.update(cx, |minimap, cx| minimap.sync_editor(cx)) {
                log::debug!("minimap released before restyling: {error:#}");
            }
        });
    }

    fn set_height(&mut self, height: Pixels, cx: &mut Context<Self>) {
        self.height = height;
        self.sync_editor(cx);
    }

    fn visible_lines(&self) -> Option<Range<usize>> {
        let viewport = self.viewport?;
        (self.height > px(0.))
            .then(|| Geometry::with_height(self.height, &viewport).visible_lines())
    }

    fn wanted_lines(&self) -> Option<Range<usize>> {
        let visible = self.visible_lines()?;
        let line_count = self.viewport?.line_count;
        let margin = visible.len();
        Some(visible.start.saturating_sub(margin)..(visible.end + margin).min(line_count))
    }

    fn slider_opacity(&self) -> f32 {
        if self.drag.is_some() {
            SLIDER_DRAG_OPACITY
        } else if self.hovered {
            SLIDER_HOVER_OPACITY
        } else {
            SLIDER_OPACITY
        }
    }
}

pub(crate) fn render_minimap(minimap: &Entity<Minimap>) -> impl IntoElement {
    minimap
        .clone()
        .cached(StyleRefinement::default().w(WIDTH).h_full().flex_shrink_0())
}

impl Render for Minimap {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let slider_color = cx.theme().foreground.opacity(self.slider_opacity());
        div().size_full().child(MinimapElement::new(
            cx.entity(),
            self.chunks.rows(),
            self.viewport,
            self.height,
            slider_color,
        ))
    }
}
