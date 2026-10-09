use std::ops::Deref;
use std::path::PathBuf;

use gpui_kit::*;

use super::workspace_context::WorkspaceContext;

#[derive(Clone, Copy)]
pub(crate) struct Panel {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) order: i32,
    pub(crate) build: fn(&WorkspaceContext, &mut Window, &mut App) -> AnyPanelView,
}

pub(crate) enum PanelEvent {
    OpenFile(PathBuf),
    OpenFileAt {
        path: PathBuf,
        line: usize,
        column: usize,
    },
    PathRenamed {
        from: PathBuf,
        to: PathBuf,
    },
    PathDeleted(PathBuf),
}

pub(crate) trait PanelView: Render + Focusable + EventEmitter<PanelEvent> {
    fn badge(&self, _: &App) -> usize {
        0
    }

    fn focus_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focus_handle = self.focus_handle(cx);
        window.focus(&focus_handle, cx);
    }
}

pub(crate) trait PanelViewHandle {
    fn view(&self) -> AnyView;
    fn focus_handle(&self, cx: &App) -> FocusHandle;
    fn focus(&self, window: &mut Window, cx: &mut App);
    fn badge(&self, cx: &App) -> usize;
    fn subscribe(
        &self,
        window: &mut Window,
        cx: &mut App,
        on_event: Box<dyn Fn(&PanelEvent, &mut Window, &mut App)>,
    ) -> Subscription;
}

impl<T: PanelView> PanelViewHandle for Entity<T> {
    fn view(&self) -> AnyView {
        self.clone().into()
    }

    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.read(cx).focus_handle(cx)
    }

    fn focus(&self, window: &mut Window, cx: &mut App) {
        self.update(cx, |view, cx| view.focus_panel(window, cx));
    }

    fn badge(&self, cx: &App) -> usize {
        self.read(cx).badge(cx)
    }

    fn subscribe(
        &self,
        window: &mut Window,
        cx: &mut App,
        on_event: Box<dyn Fn(&PanelEvent, &mut Window, &mut App)>,
    ) -> Subscription {
        window.subscribe(self, cx, move |_, event: &PanelEvent, window, cx| {
            on_event(event, window, cx)
        })
    }
}

pub(crate) struct AnyPanelView(Box<dyn PanelViewHandle>);

impl<T: PanelView> From<Entity<T>> for AnyPanelView {
    fn from(view: Entity<T>) -> Self {
        Self(Box::new(view))
    }
}

impl Deref for AnyPanelView {
    type Target = dyn PanelViewHandle;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
