mod mode;
mod preview;
mod refresh;
mod view;

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use gpui_kit::component::Rope;
use gpui_kit::*;

use super::formats::is_svg;
use super::settings::{SVG_DEFAULT_MODE, SvgMode};
use crate::editor::TEXT_VIEWER;
use crate::files::settings::SaveFormat;
use crate::plugins::{
    AnyFileView, FileView, FileViewEvent, FileViewer, GutterMarker, OpenRequest, Snapshot,
};
use crate::settings::Settings;
use preview::SvgPreview;

const REFRESH_DELAY: Duration = Duration::from_millis(150);

pub(super) const SVG_VIEWER: FileViewer = FileViewer {
    priority: 10,
    handles: is_svg,
    prepare: TEXT_VIEWER.prepare,
    open,
};

fn open(request: OpenRequest, window: &mut Window, cx: &mut App) -> Result<AnyFileView> {
    let code = (TEXT_VIEWER.open)(request, window, cx)?;
    let viewer = cx.new(|cx| SvgViewer::new(code, cx));
    Ok(viewer.into())
}

pub(super) struct SvgViewer {
    code: AnyFileView,
    preview: SvgPreview,
    preview_stale: bool,
    mode: SvgMode,
    focus_handle: FocusHandle,
    _refresh: Task<()>,
    _code_events: Subscription,
    _release: Subscription,
    _settings: Subscription,
}

impl EventEmitter<FileViewEvent> for SvgViewer {}

impl SvgViewer {
    fn new(code: AnyFileView, cx: &mut Context<Self>) -> Self {
        let viewer = cx.weak_entity();
        let code_events = code.subscribe(
            cx,
            Box::new(move |event: &FileViewEvent, cx: &mut App| {
                let edited = matches!(event, FileViewEvent::Edited);
                viewer
                    .update(cx, |viewer, cx| viewer.handle_code_event(edited, cx))
                    .ok();
            }),
        );
        let release = cx.on_release(|viewer, cx| viewer.preview.release(cx));
        let settings = cx.observe_global::<Settings>(|_, cx| cx.notify());
        let mut viewer = Self {
            code,
            preview: SvgPreview::default(),
            preview_stale: true,
            mode: SVG_DEFAULT_MODE.get(cx),
            focus_handle: cx.focus_handle(),
            _refresh: Task::ready(()),
            _code_events: code_events,
            _release: release,
            _settings: settings,
        };
        viewer.refresh_if_visible(Duration::ZERO, cx);
        viewer
    }

    fn handle_code_event(&mut self, edited: bool, cx: &mut Context<Self>) {
        if edited {
            cx.emit(FileViewEvent::Edited);
            self.preview_stale = true;
            self.refresh_if_visible(REFRESH_DELAY, cx);
        } else {
            cx.emit(FileViewEvent::StatusChanged);
        }
    }

    fn refresh_if_visible(&mut self, delay: Duration, cx: &mut Context<Self>) {
        if self.preview_stale && self.mode.shows_preview() {
            self.preview_stale = false;
            self.schedule_refresh(delay, cx);
        }
    }

    fn set_mode(&mut self, mode: SvgMode, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = mode;
        self.refresh_if_visible(Duration::ZERO, cx);
        let focus_handle = self.focus_handle(cx);
        window.focus(&focus_handle, cx);
        cx.notify();
    }
}

impl FileView for SvgViewer {
    fn reload(
        &mut self,
        bytes: Vec<u8>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.code.reload(bytes, window, cx)
    }

    fn differs_from_saved(&self, cx: &App) -> bool {
        self.code.differs_from_saved(cx)
    }

    fn snapshot(&mut self, cx: &mut Context<Self>) -> Option<Result<Snapshot>> {
        self.code.snapshot(cx)
    }

    fn prepare_save(&mut self, format: &SaveFormat, window: &mut Window, cx: &mut Context<Self>) {
        self.code.prepare_save(format, window, cx);
    }

    fn unsaved_text(&self, cx: &App) -> Option<Rope> {
        self.code.unsaved_text(cx)
    }

    fn restore_unsaved(&mut self, text: Rope, window: &mut Window, cx: &mut Context<Self>) {
        self.code.restore_unsaved(text, window, cx);
    }

    fn set_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.code.set_path(path, cx);
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        self.code.status_items(cx)
    }

    fn running_work(&self, cx: &App) -> Option<&'static str> {
        self.code.running_work(cx)
    }

    fn go_to(&mut self, line: usize, column: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.mode.shows_code() {
            self.set_mode(SvgMode::Split, window, cx);
        }
        self.code.go_to(line, column, window, cx);
    }

    fn text(&self, cx: &App) -> Option<Rope> {
        self.code.text(cx)
    }

    fn set_gutter_markers(&mut self, markers: Vec<GutterMarker>, cx: &mut Context<Self>) {
        self.code.set_gutter_markers(markers, cx);
    }
}

impl Focusable for SvgViewer {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match self.mode {
            SvgMode::Preview => self.focus_handle.clone(),
            SvgMode::Code | SvgMode::Split => self.code.focus_handle(cx),
        }
    }
}
