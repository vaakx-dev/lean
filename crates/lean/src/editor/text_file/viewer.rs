use std::path::Path;

use anyhow::Result;
use gpui_kit::component::highlighter::{LanguageRegistry, SyntaxHighlighter};
use gpui_kit::*;

use super::TextFile;
use crate::editor::binary_file::BinaryFile;
use crate::editor::file_text::normalize_line_endings;
use crate::files::Language;
use crate::files::settings::highlight_limit_bytes;
use crate::plugins::{AnyFileView, FileViewer, OpenRequest};

pub(crate) const TEXT_VIEWER: FileViewer = FileViewer {
    priority: i32::MIN,
    handles: handles_any,
    prepare: Some(prepare),
    open,
};

pub(super) fn language_for(path: &Path, content_length: usize) -> Language {
    if content_length > highlight_limit_bytes() {
        Language::PLAIN
    } else {
        Language::for_path(path)
    }
}

fn handles_any(_: &Path) -> bool {
    true
}

fn prepare(path: &Path, bytes: &[u8]) {
    if std::str::from_utf8(bytes).is_err() {
        return;
    }
    let id = language_for(path, bytes.len()).id();
    if LanguageRegistry::singleton().language(id).is_some() {
        drop(SyntaxHighlighter::new(id));
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
