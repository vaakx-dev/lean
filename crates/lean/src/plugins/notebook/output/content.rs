use std::sync::Arc;

use gpui_kit::{RenderImage, SharedString};
use serde_json::Value;

use super::ansi::strip_ansi;
use super::stream::StreamText;
use crate::plugins::notebook::document::join_text;

pub(super) enum OutputContent {
    Text(SharedString),
    Stream {
        text: StreamText,
        stderr: bool,
    },
    Error {
        title: SharedString,
        traceback: SharedString,
    },
    Image(Arc<RenderImage>),
    Markdown(SharedString),
    Html(SharedString),
    Unsupported(SharedString),
}

pub(super) fn data_content(data: &Value, max_lines: usize) -> OutputContent {
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

pub(super) fn error_content(json: &Value, max_lines: usize) -> OutputContent {
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

fn limit_lines(text: &str, max_lines: usize) -> SharedString {
    let text = text.trim_end_matches('\n');
    let line_count = text.lines().count();
    if line_count <= max_lines {
        return text.to_string().into();
    }
    truncated(text.lines().take(max_lines), line_count - max_lines)
}

pub(super) fn truncated<'a>(kept: impl Iterator<Item = &'a str>, hidden: usize) -> SharedString {
    let kept: Vec<&str> = kept.collect();
    format!("{}\n… {hidden} more lines", kept.join("\n")).into()
}
