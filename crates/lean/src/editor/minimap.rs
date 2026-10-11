mod chunks;
mod element;
mod geometry;
mod interaction;
mod refresh;
mod rows;
mod style;
mod viewport;

use std::ops::Range;

use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::*;

use crate::editor::settings::MinimapSlider;
use crate::settings::Settings;
use chunks::Chunks;
use element::MinimapElement;
use geometry::Geometry;
use interaction::Drag;
use refresh::{Refresh, Styling};
use style::{MinimapStyle, minimap_width};
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
    refresh_pending: bool,
    style: MinimapStyle,
    hovered: bool,
    drag: Option<Drag>,
    _subscriptions: [Subscription; 4],
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
            cx.observe_global::<Settings>(|minimap, cx| minimap.sync_style(cx)),
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
            refresh_pending: false,
            style: MinimapStyle::read(cx),
            hovered: false,
            drag: None,
            _subscriptions: subscriptions,
        }
    }

    fn sync_editor(&mut self, cx: &mut Context<Self>) {
        let editor = self.editor.read(cx);
        let viewport = EditorViewport::read(editor);
        let styling = Styling::read(editor);
        let restyled = self.refresh.sync_styling(styling);
        let moved = viewport != self.viewport;
        if moved {
            self.viewport = viewport;
            if let Some(viewport) = viewport {
                self.chunks.drop_past(viewport.line_count);
            }
            cx.notify();
        }
        if moved || restyled {
            self.request_refresh(cx);
        }
    }

    fn sync_editor_later(&self, cx: &mut Context<Self>) {
        let minimap = cx.entity().downgrade();
        cx.defer(move |cx| {
            minimap
                .update(cx, |minimap, cx| minimap.sync_editor(cx))
                .ok();
        });
    }

    fn sync_style(&mut self, cx: &mut Context<Self>) {
        let style = MinimapStyle::read(cx);
        if style == self.style {
            return;
        }
        if style.changes_rows(&self.style) {
            self.refresh.restyle();
        }
        self.style = style;
        self.request_refresh(cx);
        cx.notify();
    }

    fn request_refresh(&mut self, cx: &mut Context<Self>) {
        if !self.refresh_pending {
            self.refresh_pending = true;
            cx.notify();
        }
    }

    fn painted(&mut self, height: Pixels, cx: &mut Context<Self>) {
        if height != self.height {
            self.height = height;
            self.sync_editor(cx);
        }
        self.refresh_pending = false;
        self.refresh_if_needed(cx);
    }

    fn visible_lines(&self) -> Option<Range<usize>> {
        let viewport = self.viewport?;
        (self.height > px(0.)).then(|| {
            Geometry::with_height(self.height, &viewport, self.style.line_height).visible_lines()
        })
    }

    fn wanted_lines(&self) -> Option<Range<usize>> {
        let visible = self.visible_lines()?;
        let line_count = self.viewport?.line_count;
        let margin = visible.len();
        Some(visible.start.saturating_sub(margin)..(visible.end + margin).min(line_count))
    }

    fn slider_color(&self, cx: &App) -> Option<Hsla> {
        let active = self.drag.is_some() || self.hovered;
        let shown = match self.style.slider {
            MinimapSlider::Always => true,
            MinimapSlider::Hover => active,
            MinimapSlider::Never => false,
        };
        shown.then(|| cx.theme().foreground.opacity(self.slider_opacity()))
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

pub(crate) fn render_minimap(minimap: &Entity<Minimap>, cx: &App) -> impl IntoElement {
    minimap.clone().cached(
        StyleRefinement::default()
            .w(minimap_width(cx))
            .h_full()
            .flex_shrink_0(),
    )
}

impl Render for Minimap {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(MinimapElement {
            minimap: cx.entity(),
            rows: self.chunks.rows(),
            viewport: self.viewport,
            height: self.height,
            style: self.style,
            slider_color: self.slider_color(cx),
            refresh_pending: self.refresh_pending,
        })
    }
}
