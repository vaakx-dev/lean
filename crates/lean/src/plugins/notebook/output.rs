mod ansi;
mod stream;
mod view;

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui_kit::{App, Image, ImageFormat, SharedString};
use serde_json::Value;

use super::document::join_text;
use super::settings::OUTPUT_MAX_LINES;
use ansi::strip_ansi;
use stream::StreamText;

const IMAGE_FORMATS: [ImageFormat; 2] = [ImageFormat::Png, ImageFormat::Jpeg];

pub(crate) struct Output {
    pub(crate) json: Value,
    content: OutputContent,
}

enum OutputContent {
    Text(SharedString),
    Stream {
        text: StreamText,
        stderr: bool,
    },
    Error {
        title: SharedString,
        traceback: SharedString,
    },
    Image(Arc<Image>),
    Markdown(SharedString),
    Html(SharedString),
    Unsupported(SharedString),
}

impl Output {
    pub(crate) fn new(json: Value, cx: &App) -> Output {
        let max_lines = OUTPUT_MAX_LINES.get(cx).max(1) as usize;
        let content = OutputContent::from_json(&json, max_lines);
        Output { json, content }
    }

    pub(crate) fn is_error(&self) -> bool {
        matches!(self.content, OutputContent::Error { .. })
    }

    pub(crate) fn merge_stream(&mut self, next: &Value) -> bool {
        let (Some(name), Some(next_name)) = (stream_name(&self.json), stream_name(next)) else {
            return false;
        };
        if name != next_name {
            return false;
        }
        let OutputContent::Stream { text, .. } = &mut self.content else {
            return false;
        };
        let chunk = next.get("text").map(join_text).unwrap_or_default();
        text.push(&chunk);
        append_text(&mut self.json, &chunk);
        true
    }
}

fn append_text(json: &mut Value, chunk: &str) {
    if let Some(Value::String(text)) = json.get_mut("text") {
        text.push_str(chunk);
        return;
    }
    let mut text = json.get("text").map(join_text).unwrap_or_default();
    text.push_str(chunk);
    json["text"] = Value::String(text);
}

impl OutputContent {
    fn from_json(json: &Value, max_lines: usize) -> OutputContent {
        match json.get("output_type").and_then(Value::as_str) {
            Some("stream") => OutputContent::Stream {
                text: StreamText::new(
                    &json.get("text").map(join_text).unwrap_or_default(),
                    max_lines,
                ),
                stderr: stream_name(json) == Some("stderr"),
            },
            Some("error") => error_content(json, max_lines),
            Some("execute_result" | "display_data") => match json.get("data") {
                Some(data) => image_content(data, max_lines),
                None => OutputContent::Unsupported("Empty output".into()),
            },
            other => OutputContent::Unsupported(
                format!("Unsupported output type {}", other.unwrap_or("unknown")).into(),
            ),
        }
    }
}

fn image_content(data: &Value, max_lines: usize) -> OutputContent {
    match decode_image(data) {
        Some(image) => OutputContent::Image(image),
        None => data_content(data, max_lines),
    }
}

fn decode_image(data: &Value) -> Option<Arc<Image>> {
    IMAGE_FORMATS.into_iter().find_map(|format| {
        let encoded: String = join_text(data.get(format.mime_type())?)
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
    })
}

fn data_content(data: &Value, max_lines: usize) -> OutputContent {
    if let Some(html) = data.get("text/html") {
        return OutputContent::Html(join_text(html).into());
    }
    if let Some(markdown) = data.get("text/markdown") {
        return OutputContent::Markdown(join_text(markdown).into());
    }
    if let Some(text) = data.get("text/plain") {
        return OutputContent::Text(limit_lines(&strip_ansi(&join_text(text)), max_lines));
    }
    let mimes = data
        .as_object()
        .map(|data| data.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    OutputContent::Unsupported(format!("No renderer for {mimes}").into())
}

fn error_content(json: &Value, max_lines: usize) -> OutputContent {
    let name = json.get("ename").and_then(Value::as_str).unwrap_or("Error");
    let value = json.get("evalue").and_then(Value::as_str).unwrap_or("");
    let traceback = json
        .get("traceback")
        .and_then(Value::as_array)
        .map(|lines| {
            lines
                .iter()
                .filter_map(Value::as_str)
                .map(strip_ansi)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    OutputContent::Error {
        title: format!("{name}: {value}").into(),
        traceback: limit_lines(&traceback, max_lines),
    }
}

fn stream_name(json: &Value) -> Option<&str> {
    if json.get("output_type").and_then(Value::as_str) != Some("stream") {
        return None;
    }
    json.get("name").and_then(Value::as_str)
}

fn limit_lines(text: &str, max_lines: usize) -> SharedString {
    let text = text.trim_end_matches('\n');
    let line_count = text.lines().count();
    if line_count <= max_lines {
        return text.to_string().into();
    }
    truncated(text.lines().take(max_lines), line_count - max_lines)
}

fn truncated<'a>(kept: impl Iterator<Item = &'a str>, hidden: usize) -> SharedString {
    let kept: Vec<&str> = kept.collect();
    format!("{}\n… {hidden} more lines", kept.join("\n")).into()
}
