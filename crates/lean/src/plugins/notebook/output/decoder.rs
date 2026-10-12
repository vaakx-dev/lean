use std::slice;
use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui_kit::{App, Image, ImageFormat, RenderImage, SvgRenderer};
use serde_json::Value;

use super::content::{OutputContent, data_content, error_content};
use super::stream::StreamText;
use super::{Output, stream_name};
use crate::plugins::notebook::document::join_text;
use crate::plugins::notebook::settings::OUTPUT_MAX_LINES;

const IMAGE_FORMATS: [ImageFormat; 2] = [ImageFormat::Png, ImageFormat::Jpeg];

#[derive(Clone)]
pub(crate) struct OutputDecoder {
    max_lines: usize,
    renderer: SvgRenderer,
}

impl OutputDecoder {
    pub(crate) fn new(cx: &App) -> Self {
        Self {
            max_lines: OUTPUT_MAX_LINES.get(cx).max(1) as usize,
            renderer: cx.svg_renderer(),
        }
    }

    pub(crate) fn decode(&self, json: Value) -> Output {
        let content = self.content(&json);
        Output {
            json: Arc::new(json),
            content,
        }
    }

    pub(super) fn content(&self, json: &Value) -> OutputContent {
        match json.get("output_type").and_then(Value::as_str) {
            Some("stream") => OutputContent::Stream {
                text: StreamText::new(
                    &json.get("text").map(join_text).unwrap_or_default(),
                    self.max_lines,
                ),
                stderr: stream_name(json) == Some("stderr"),
            },
            Some("error") => error_content(json, self.max_lines),
            Some("execute_result" | "display_data") => match json.get("data") {
                Some(data) => match self.image(data) {
                    Some(image) => OutputContent::Image(image),
                    None => data_content(data, self.max_lines),
                },
                None => OutputContent::Unsupported("Empty output".into()),
            },
            other => OutputContent::Unsupported(
                format!("Unsupported output type {}", other.unwrap_or("unknown")).into(),
            ),
        }
    }

    fn image(&self, data: &Value) -> Option<Arc<RenderImage>> {
        IMAGE_FORMATS.into_iter().find_map(|format| {
            let encoded = base64_bytes(data.get(format.mime_type())?);
            let bytes = STANDARD
                .decode(encoded)
                .inspect_err(|error| log::warn!("failed to decode a notebook image: {error}"))
                .ok()?;
            Image::from_bytes(format, bytes)
                .to_image_data(self.renderer.clone())
                .inspect_err(|error| log::warn!("failed to decode a notebook image: {error:#}"))
                .ok()
        })
    }
}

fn base64_bytes(value: &Value) -> Vec<u8> {
    let lines = match value {
        Value::Array(lines) => lines.as_slice(),
        _ => slice::from_ref(value),
    };
    lines
        .iter()
        .filter_map(Value::as_str)
        .flat_map(str::bytes)
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect()
}
