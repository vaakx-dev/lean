mod document;
mod element;
mod geometry;
mod interaction;
mod refresh;
mod viewport;

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::input::{EditorState, InputEvent};
use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::*;

use document::Document;
use element::MinimapElement;
use interaction::Drag;
use refresh::{EDIT_REFRESH_DELAY, Syntax};
use viewport::EditorViewport;

const MINIMAP_WIDTH: Pixels = px(100.);
const SLIDER_OPACITY: f32 = 0.06;
const SLIDER_HOVER_OPACITY: f32 = 0.12;
const SLIDER_DRAG_OPACITY: f32 = 0.18;

pub(crate) struct Minimap {
    editor: Entity<EditorState>,
    syntax: Syntax,
    document: Rc<Document>,
    viewport: Option<EditorViewport>,
    hovered: bool,
    drag: Option<Drag>,
    _refresh_task: Task<()>,
    _subscriptions: [Subscription; 3],
}

impl Minimap {
    pub(crate) fn new(editor: Entity<EditorState>, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.observe(&editor, |minimap, editor, cx| {
                minimap.sync_viewport(&editor, cx)
            }),
            cx.subscribe(&editor, |minimap, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    minimap.schedule_refresh(EDIT_REFRESH_DELAY, cx);
                }
            }),
            cx.observe_global::<Theme>(|minimap, cx| minimap.schedule_refresh(Duration::ZERO, cx)),
        ];
        let state = editor.read(cx);
        let syntax = Syntax::new(state.language_name());
        let viewport = EditorViewport::read(state);
        let mut minimap = Self {
            editor,
            syntax,
            document: Rc::default(),
            viewport,
            hovered: false,
            drag: None,
            _refresh_task: Task::ready(()),
            _subscriptions: subscriptions,
        };
        minimap.schedule_refresh(Duration::ZERO, cx);
        minimap
    }

    fn sync_viewport(&mut self, editor: &Entity<EditorState>, cx: &mut Context<Self>) {
        let viewport = EditorViewport::read(editor.read(cx));
        if viewport != self.viewport {
            self.viewport = viewport;
            cx.notify();
        }
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

impl Render for Minimap {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let slider_color = cx.theme().foreground.opacity(self.slider_opacity());
        div()
            .w(MINIMAP_WIDTH)
            .h_full()
            .flex_shrink_0()
            .child(MinimapElement::new(
                cx.entity(),
                self.document.clone(),
                self.viewport,
                slider_color,
            ))
    }
}
