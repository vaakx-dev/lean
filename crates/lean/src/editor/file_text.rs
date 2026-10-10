use std::hash::{DefaultHasher, Hasher as _};

use anyhow::{Result, anyhow};
use gpui_kit::component::Rope;
use ropey::extra::esoterica::ropes_are_instances;

use crate::files::settings::LineEndings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineEnding {
    crlf: bool,
    mixed: bool,
}

impl LineEnding {
    const LF: Self = Self {
        crlf: false,
        mixed: false,
    };
    const CRLF: Self = Self {
        crlf: true,
        mixed: false,
    };

    fn detect(text: &str) -> Self {
        let (mut lf, mut crlf) = (0usize, 0usize);
        for (index, _) in text.match_indices('\n') {
            if text.as_bytes()[..index].last() == Some(&b'\r') {
                crlf += 1;
            } else {
                lf += 1;
            }
        }
        Self {
            crlf: crlf > lf,
            mixed: crlf > 0 && lf > 0,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match (self.crlf, self.mixed) {
            (false, false) => "LF",
            (true, false) => "CRLF",
            (false, true) => "Mixed (LF)",
            (true, true) => "Mixed (CRLF)",
        }
    }

    pub(crate) fn for_save(self, policy: LineEndings) -> Self {
        match policy {
            LineEndings::Preserve => Self {
                mixed: false,
                ..self
            },
            LineEndings::Lf => Self::LF,
            LineEndings::Crlf => Self::CRLF,
        }
    }

    pub(crate) fn encode(self, text: String) -> Vec<u8> {
        let text = if text.contains("\r\n") {
            text.replace("\r\n", "\n")
        } else {
            text
        };
        if self.crlf {
            text.replace('\n', "\r\n").into_bytes()
        } else {
            text.into_bytes()
        }
    }
}

pub(crate) fn decode_text(bytes: Vec<u8>) -> Result<(String, LineEnding)> {
    let text = String::from_utf8(bytes).map_err(|_| anyhow!("it is not a UTF-8 text file"))?;
    Ok(normalize_line_endings(text))
}

pub(super) fn normalize_line_endings(text: String) -> (String, LineEnding) {
    let ending = LineEnding::detect(&text);
    if ending.crlf || ending.mixed {
        (text.replace("\r\n", "\n"), ending)
    } else {
        (text, ending)
    }
}

pub(super) fn content_hash(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    hasher.finish()
}

pub(super) fn same_text(text: &Rope, saved: &Rope) -> bool {
    ropes_are_instances(text, saved) || (text.len() == saved.len() && text == saved)
}
