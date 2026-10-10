use std::path::Path;

use gpui_kit::component::highlighter::{LanguageRegistry, SyntaxHighlighter};

use crate::files::Language;
use crate::files::settings::highlight_limit_bytes;

pub(super) fn language_for(path: &Path, content_length: usize) -> Language {
    if content_length > highlight_limit_bytes() {
        Language::PLAIN
    } else {
        Language::for_path(path)
    }
}

pub(in crate::editor) fn warm_highlighter(path: &Path, content_length: usize) {
    let id = language_for(path, content_length).id();
    if LanguageRegistry::singleton().language(id).is_some() {
        drop(SyntaxHighlighter::new(id));
    }
}
