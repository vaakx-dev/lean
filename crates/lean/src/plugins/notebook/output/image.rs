use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui_kit::{AppContext as _, Context, Image, ImageFormat};
use serde_json::Value;

use crate::plugins::notebook::Notebook;
use crate::plugins::notebook::document::join_text;

const IMAGE_FORMATS: [(&str, ImageFormat); 2] = [
    ("image/png", ImageFormat::Png),
    ("image/jpeg", ImageFormat::Jpeg),
];

#[derive(Clone)]
pub(super) enum ImageState {
    Loading,
    Ready(Arc<Image>),
    Failed,
}

pub(super) struct ImageOutput {
    state: Rc<RefCell<ImageState>>,
}

impl ImageOutput {
    pub(super) fn load(data: &Value, cx: &mut Context<Notebook>) -> Option<Self> {
        let candidates: Vec<(String, ImageFormat)> = IMAGE_FORMATS
            .into_iter()
            .filter_map(|(mime, format)| Some((join_text(data.get(mime)?), format)))
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let state = Rc::new(RefCell::new(ImageState::Loading));
        let shared = Rc::clone(&state);
        cx.spawn(async move |notebook, cx| {
            let image = cx
                .background_spawn(async move { decode_first(&candidates) })
                .await;
            *shared.borrow_mut() = match image {
                Some(image) => ImageState::Ready(image),
                None => ImageState::Failed,
            };
            if let Err(error) = notebook.update(cx, |_, cx| cx.notify()) {
                log::debug!("notebook closed before an image finished decoding: {error:#}");
            }
        })
        .detach();
        Some(Self { state })
    }

    pub(super) fn state(&self) -> ImageState {
        self.state.borrow().clone()
    }
}

fn decode_first(candidates: &[(String, ImageFormat)]) -> Option<Arc<Image>> {
    candidates
        .iter()
        .find_map(|(encoded, format)| decode_image(encoded, *format))
}

fn decode_image(encoded: &str, format: ImageFormat) -> Option<Arc<Image>> {
    let encoded: String = encoded
        .chars()
        .filter(|char| !char.is_whitespace())
        .collect();
    match STANDARD.decode(encoded) {
        Ok(bytes) => Some(Arc::new(Image::from_bytes(format, bytes))),
        Err(error) => {
            log::warn!("failed to decode a notebook image: {error}");
            None
        }
    }
}
