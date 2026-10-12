use gpui_kit::component::status_bar::StatusBar;
use gpui_kit::component::{ActiveTheme as _, Theme, h_flex};
use gpui_kit::*;

use super::Workspace;
use crate::editor::{EditorArea, EditorAreaEvent};
use crate::plugins::{WorkspaceContext, status_items};

const STATUS_BAR_HEIGHT: Pixels = px(28.);

pub(super) struct WorkspaceStatusBar {
    plugin_items: Vec<AnyView>,
    editor_area: Entity<EditorArea>,
    _subscriptions: [Subscription; 2],
}

impl WorkspaceStatusBar {
    pub(super) fn new(
        context: &WorkspaceContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let plugin_items = status_items(cx)
            .to_vec()
            .into_iter()
            .map(|item| (item.build)(context, window, cx))
            .collect();
        let subscriptions = [
            cx.subscribe(&context.editor_area, |_, _, event: &EditorAreaEvent, cx| {
                if matches!(
                    event,
                    EditorAreaEvent::StatusChanged | EditorAreaEvent::ActiveEditorChanged
                ) {
                    cx.notify();
                }
            }),
            cx.observe(&context.toolchains, |_, _, cx| cx.notify()),
        ];
        Self {
            plugin_items,
            editor_area: context.editor_area.clone(),
            _subscriptions: subscriptions,
        }
    }
}

impl Render for WorkspaceStatusBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx: &App = cx;
        let editor_items = self.editor_area.read(cx).status_items(cx);
        StatusBar::new()
            .size_full()
            .items_stretch()
            .py_0p5()
            .border_t_0()
            .bg(cx.theme().background)
            .text_sm()
            .left(region().children(self.plugin_items.iter().cloned()))
            .right(region().children(editor_items))
    }
}

impl Workspace {
    pub(super) fn render_status_bar(&self) -> impl IntoElement + use<> {
        self.status_bar.clone().cached(
            StyleRefinement::default()
                .w_full()
                .h(STATUS_BAR_HEIGHT)
                .flex_shrink_0(),
        )
    }
}

fn region() -> Div {
    h_flex().h_full().gap_1()
}

pub(crate) fn status_item(id: &'static str, theme: &Theme) -> Stateful<Div> {
    h_flex()
        .id(id)
        .h_full()
        .px_1p5()
        .gap_1()
        .rounded(theme.radius)
        .hover(|style| style.bg(theme.accent).text_color(theme.foreground))
}
