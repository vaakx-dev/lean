mod ansi;
mod content;
mod decoder;
mod stream;
mod view;

use std::sync::Arc;

use gpui_kit::{App, RenderImage};
use serde_json::Value;

use super::document::join_text;
use content::OutputContent;

pub(crate) use decoder::OutputDecoder;

pub(crate) struct Output {
    pub(crate) json: Arc<Value>,
    content: OutputContent,
}

impl Output {
    pub(crate) fn is_error(&self) -> bool {
        matches!(self.content, OutputContent::Error { .. })
    }

    pub(crate) fn relimit(&mut self, decoder: &OutputDecoder) {
        if !matches!(self.content, OutputContent::Image(_)) {
            self.content = decoder.content(&self.json);
        }
    }

    pub(crate) fn merge_stream(&mut self, next: &Output) -> bool {
        let (Some(name), Some(next_name)) = (stream_name(&self.json), stream_name(&next.json))
        else {
            return false;
        };
        if name != next_name {
            return false;
        }
        let OutputContent::Stream { text, .. } = &mut self.content else {
            return false;
        };
        let chunk = next.json.get("text").map(join_text).unwrap_or_default();
        text.push(&chunk);
        append_text(Arc::make_mut(&mut self.json), &chunk);
        true
    }

    fn into_image(self) -> Option<Arc<RenderImage>> {
        match self.content {
            OutputContent::Image(image) => Some(image),
            _ => None,
        }
    }
}

pub(crate) fn release_outputs(outputs: impl IntoIterator<Item = Output>, cx: &mut App) {
    let images: Vec<Arc<RenderImage>> =
        outputs.into_iter().filter_map(Output::into_image).collect();
    if images.is_empty() {
        return;
    }
    cx.defer(move |cx| {
        for image in images {
            cx.drop_image(image, None);
        }
    });
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

fn stream_name(json: &Value) -> Option<&str> {
    if json.get("output_type").and_then(Value::as_str) != Some("stream") {
        return None;
    }
    json.get("name").and_then(Value::as_str)
}
