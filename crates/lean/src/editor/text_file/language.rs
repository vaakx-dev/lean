use std::path::Path;

use gpui_kit::component::highlighter::{LanguageRegistry, SyntaxHighlighter};

use crate::files::Language;

const HIGHLIGHT_SIZE_LIMIT: usize = 4 * 1024 * 1024;

pub(super) fn language_for(path: &Path, content_length: usize) -> Language {
    if content_length > HIGHLIGHT_SIZE_LIMIT {
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
