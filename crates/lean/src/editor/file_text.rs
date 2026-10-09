use std::hash::{DefaultHasher, Hasher as _};

use anyhow::{Result, anyhow};
use gpui_kit::component::Rope;
use ropey::extra::esoterica::ropes_are_instances;

const COMPARE_LIMIT: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    fn detect(text: &str) -> Self {
        match text.find('\n') {
            Some(index) if text[..index].ends_with('\r') => Self::Crlf,
            _ => Self::Lf,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
        }
    }

    pub(crate) fn encode(self, text: String) -> Vec<u8> {
        match self {
            Self::Lf => text.into_bytes(),
            Self::Crlf => text
                .replace("\r\n", "\n")
                .replace('\n', "\r\n")
                .into_bytes(),
        }
    }
}

pub(crate) fn decode_text(bytes: Vec<u8>) -> Result<(String, LineEnding)> {
    let text = String::from_utf8(bytes).map_err(|_| anyhow!("it is not a UTF-8 text file"))?;
    Ok(normalize_line_endings(text))
}

pub(super) fn normalize_line_endings(text: String) -> (String, LineEnding) {
    match LineEnding::detect(&text) {
        LineEnding::Lf => (text, LineEnding::Lf),
        LineEnding::Crlf => (text.replace("\r\n", "\n"), LineEnding::Crlf),
    }
}

pub(super) fn content_hash(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

pub(super) fn same_text(text: &Rope, saved: &Rope) -> bool {
    ropes_are_instances(text, saved)
        || (text.len() == saved.len() && text.len() <= COMPARE_LIMIT && text == saved)
}
