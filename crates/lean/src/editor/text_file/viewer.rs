use std::path::Path;

use anyhow::Result;
use gpui_kit::*;

use super::TextFile;
use super::language::warm_highlighter;
use crate::editor::binary_file::BinaryFile;
use crate::editor::file_text::normalize_line_endings;
use crate::plugins::{AnyFileView, FileViewer, OpenRequest};

pub(crate) const TEXT_VIEWER: FileViewer = FileViewer {
    priority: i32::MIN,
    handles: handles_any,
    prepare: Some(prepare),
    open,
};

fn handles_any(_: &Path) -> bool {
    true
}

fn prepare(path: &Path, bytes: &[u8]) {
    if std::str::from_utf8(bytes).is_ok() {
        warm_highlighter(path, bytes.len());
    }
}

fn open(request: OpenRequest, window: &mut Window, cx: &mut App) -> Result<AnyFileView> {
    let path = request.path;
    let text = match String::from_utf8(request.bytes) {
        Ok(text) => text,
        Err(error) => return Ok(BinaryFile::open(path, error.as_bytes().len(), cx)),
    };
    let (text, line_ending) = normalize_line_endings(text);
    let toolchains = request.toolchains;
    let file = cx.new(|cx| TextFile::new(&path, text, line_ending, toolchains, window, cx));
    Ok(file.into())
}
