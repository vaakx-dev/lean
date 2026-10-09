mod keymap;
mod navigation;
mod stage;
mod status;
mod toolbar;
mod zoom;

use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::*;

use super::formats::{detect_format, raster_format};
use crate::plugins::{AnyFileView, FileView, FileViewEvent, FileViewer, OpenRequest};
use navigation::Drag;
use zoom::Zoom;

pub(super) use keymap::bind_keys;

pub(super) const IMAGE_VIEWER: FileViewer = FileViewer {
    priority: 10,
    handles: is_image,
    prepare: None,
    open,
};

fn is_image(path: &Path) -> bool {
    raster_format(path).is_some()
}

fn open(request: OpenRequest, _: &mut Window, cx: &mut App) -> Result<AnyFileView> {
    let viewer = cx.new(|cx| ImageViewer::new(request.path, request.bytes, cx));
    Ok(viewer.into())
}

enum Decoded {
    Loading,
    Ready(Arc<RenderImage>),
    Failed(SharedString),
}

pub(super) struct ImageViewer {
    path: PathBuf,
    image: Arc<Image>,
    decoded: Decoded,
    zoom: Zoom,
    drag: Option<Drag>,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    _decoding: Task<()>,
    _release: Subscription,
}

impl EventEmitter<FileViewEvent> for ImageViewer {}

impl ImageViewer {
    fn new(path: PathBuf, bytes: Vec<u8>, cx: &mut Context<Self>) -> Self {
        let image = Arc::new(Image::from_bytes(detect_format(&path, &bytes), bytes));
        let release = cx.on_release(|viewer, cx| {
            if let Decoded::Ready(image) = mem::replace(&mut viewer.decoded, Decoded::Loading) {
                cx.drop_image(image, None);
            }
        });
        let mut viewer = Self {
            path,
            image,
            decoded: Decoded::Loading,
            zoom: Zoom::Fit,
            drag: None,
            scroll_handle: ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            _decoding: Task::ready(()),
            _release: release,
        };
        viewer.decode(cx);
        viewer
    }

    fn decode(&mut self, cx: &mut Context<Self>) {
        let image = self.image.clone();
        let renderer = cx.svg_renderer();
        self._decoding = cx.spawn(async move |viewer, cx| {
            let decoded = cx
                .background_spawn(async move { image.to_image_data(renderer) })
                .await;
            let update = viewer.update(cx, |viewer, cx| viewer.show_decoded(decoded, cx));
            if let Err(error) = update {
                log::debug!("image viewer closed before decoding finished: {error:#}");
            }
        });
    }

    fn show_decoded(&mut self, decoded: Result<Arc<RenderImage>>, cx: &mut Context<Self>) {
        let decoded = match decoded {
            Ok(image) => Decoded::Ready(image),
            Err(error) => {
                log::warn!("failed to decode {}: {error:#}", self.path.display());
                Decoded::Failed(format!("{error:#}").into())
            }
        };
        if let Decoded::Ready(previous) = mem::replace(&mut self.decoded, decoded) {
            cx.drop_image(previous, None);
        }
        cx.emit(FileViewEvent::StatusChanged);
        cx.notify();
    }

    fn rendered(&self) -> Option<&Arc<RenderImage>> {
        match &self.decoded {
            Decoded::Ready(image) => Some(image),
            Decoded::Loading | Decoded::Failed(_) => None,
        }
    }

    fn image_size(&self) -> Option<Size<DevicePixels>> {
        self.rendered().map(|image| image.size(0))
    }
}

impl FileView for ImageViewer {
    fn reload(&mut self, bytes: Vec<u8>, _: &mut Window, cx: &mut Context<Self>) -> Result<()> {
        let format = detect_format(&self.path, &bytes);
        self.image = Arc::new(Image::from_bytes(format, bytes));
        self.decode(cx);
        cx.emit(FileViewEvent::StatusChanged);
        Ok(())
    }

    fn differs_from_saved(&self, _: &App) -> bool {
        false
    }

    fn set_path(&mut self, path: &Path, _: &mut Context<Self>) {
        self.path = path.to_path_buf();
    }

    fn status_items(&self, cx: &App) -> Vec<AnyElement> {
        self.render_status_items(cx)
    }
}

impl Focusable for ImageViewer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
