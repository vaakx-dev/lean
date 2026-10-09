use std::ops::Deref;
use std::path::Path;

use anyhow::Result;
use gpui_kit::component::Rope;
use gpui_kit::component::input::GutterMarker;
use gpui_kit::*;

use super::snapshot::Snapshot;

pub(crate) enum FileViewEvent {
    Edited,
    StatusChanged,
}

pub(crate) trait FileView: Render + Focusable + EventEmitter<FileViewEvent> {
    fn reload(&mut self, bytes: Vec<u8>, window: &mut Window, cx: &mut Context<Self>)
    -> Result<()>;

    fn differs_from_saved(&self, _: &App) -> bool {
        true
    }

    fn snapshot(&mut self, _: &mut Context<Self>) -> Option<Result<Snapshot>> {
        None
    }

    fn set_path(&mut self, _: &Path, _: &mut Context<Self>) {}

    fn status_items(&self, _: &App) -> Vec<AnyElement> {
        Vec::new()
    }

    fn running_work(&self, _: &App) -> Option<&'static str> {
        None
    }

    fn go_to(&mut self, _: usize, _: usize, _: &mut Window, _: &mut Context<Self>) {}

    fn text(&self, _: &App) -> Option<Rope> {
        None
    }

    fn set_gutter_markers(&mut self, _: Vec<GutterMarker>, _: &mut Context<Self>) {}
}

pub(crate) trait FileViewHandle {
    fn view(&self) -> AnyView;
    fn focus_handle(&self, cx: &App) -> FocusHandle;
    fn reload(&self, bytes: Vec<u8>, window: &mut Window, cx: &mut App) -> Result<()>;
    fn differs_from_saved(&self, cx: &App) -> bool;
    fn snapshot(&self, cx: &mut App) -> Option<Result<Snapshot>>;
    fn set_path(&self, path: &Path, cx: &mut App);
    fn status_items(&self, cx: &App) -> Vec<AnyElement>;
    fn running_work(&self, cx: &App) -> Option<&'static str>;
    fn go_to(&self, line: usize, column: usize, window: &mut Window, cx: &mut App);
    fn text(&self, cx: &App) -> Option<Rope>;
    fn set_gutter_markers(&self, markers: Vec<GutterMarker>, cx: &mut App);
    fn subscribe(
        &self,
        cx: &mut App,
        on_event: Box<dyn Fn(&FileViewEvent, &mut App)>,
    ) -> Subscription;
}

impl<T: FileView> FileViewHandle for Entity<T> {
    fn view(&self) -> AnyView {
        self.clone().into()
    }

    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.read(cx).focus_handle(cx)
    }

    fn reload(&self, bytes: Vec<u8>, window: &mut Window, cx: &mut App) -> Result<()> {
        self.update(cx, |view, cx| view.reload(bytes, window, cx))
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        self.read(cx).differs_from_saved(cx)
    }

    fn snapshot(&self, cx: &mut App) -> Option<Result<Snapshot>> {
        self.update(cx, |view, cx| view.snapshot(cx))
    }

    fn set_path(&self, path: &Path, cx: &mut App) {
        self.update(cx, |view, cx| view.set_path(path, cx));
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        self.read(cx).status_items(cx)
    }

    fn running_work(&self, cx: &App) -> Option<&'static str> {
        self.read(cx).running_work(cx)
    }

    fn go_to(&self, line: usize, column: usize, window: &mut Window, cx: &mut App) {
        self.update(cx, |view, cx| view.go_to(line, column, window, cx));
    }

    fn text(&self, cx: &App) -> Option<Rope> {
        self.read(cx).text(cx)
    }

    fn set_gutter_markers(&self, markers: Vec<GutterMarker>, cx: &mut App) {
        self.update(cx, |view, cx| view.set_gutter_markers(markers, cx));
    }

    fn subscribe(
        &self,
        cx: &mut App,
        on_event: Box<dyn Fn(&FileViewEvent, &mut App)>,
    ) -> Subscription {
        cx.subscribe(self, move |_, event: &FileViewEvent, cx| {
            on_event(event, cx)
        })
    }
}

pub(crate) struct AnyFileView(Box<dyn FileViewHandle>);

impl<T: FileView> From<Entity<T>> for AnyFileView {
    fn from(view: Entity<T>) -> Self {
        Self(Box::new(view))
    }
}

impl Deref for AnyFileView {
    type Target = dyn FileViewHandle;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
