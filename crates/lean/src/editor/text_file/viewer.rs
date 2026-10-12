use std::path::Path;

use anyhow::Result;
use gpui_kit::component::highlighter::{LanguageRegistry, SyntaxHighlighter};
use gpui_kit::*;

use super::TextFile;
use super::decoded::{prepare_text, take_decoded};
use crate::editor::binary_file::BinaryFile;
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
    if !prepare_text(path, bytes) {
        return;
    }
    let id = language_for(path, bytes.len()).id();
    if LanguageRegistry::singleton().language(id).is_some() {
        drop(SyntaxHighlighter::new(id));
    }
}

fn open(request: OpenRequest, window: &mut Window, cx: &mut App) -> Result<AnyFileView> {
    let path = request.path;
    let size = request.bytes.len();
    let Some(decoded) = take_decoded(&path, request.bytes) else {
        return Ok(BinaryFile::open(path, size, cx));
    };
    let toolchains = request.toolchains;
    let file = cx.new(|cx| TextFile::new(path, decoded, toolchains, window, cx));
    Ok(file.into())
}
