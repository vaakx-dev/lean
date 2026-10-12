use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::editor::file_text::{Fingerprint, LineEnding, decode_str, decode_text};

const MAX_PREPARED: usize = 8;

static PREPARED: Mutex<Vec<Prepared>> = Mutex::new(Vec::new());

pub(super) struct DecodedText {
    pub(super) text: String,
    pub(super) line_ending: LineEnding,
    pub(super) fingerprint: Fingerprint,
}

impl DecodedText {
    fn new((text, line_ending): (String, LineEnding)) -> Self {
        let fingerprint = Fingerprint::of_str(&text);
        Self {
            text,
            line_ending,
            fingerprint,
        }
    }

    fn borrowed(bytes: &[u8]) -> Option<Self> {
        let text = std::str::from_utf8(bytes).ok()?;
        Some(Self::new(decode_str(text)))
    }

    fn owned(bytes: Vec<u8>) -> Option<Self> {
        decode_text(bytes).ok().map(Self::new)
    }
}

struct Prepared {
    path: PathBuf,
    source: Source,
    text: Option<DecodedText>,
}

#[derive(PartialEq, Eq)]
struct Source {
    address: usize,
    len: usize,
}

impl Source {
    fn of(bytes: &[u8]) -> Self {
        Self {
            address: bytes.as_ptr() as usize,
            len: bytes.len(),
        }
    }
}

pub(super) fn prepare_text(path: &Path, bytes: &[u8]) -> bool {
    let text = DecodedText::borrowed(bytes);
    let is_text = text.is_some();
    let mut prepared = lock();
    prepared.retain(|entry| entry.path != path);
    if prepared.len() >= MAX_PREPARED {
        prepared.remove(0);
    }
    prepared.push(Prepared {
        path: path.to_path_buf(),
        source: Source::of(bytes),
        text,
    });
    is_text
}

pub(super) fn take_decoded(path: &Path, bytes: Vec<u8>) -> Option<DecodedText> {
    let source = Source::of(&bytes);
    let prepared = {
        let mut prepared = lock();
        let index = prepared.iter().position(|entry| entry.path == path);
        index.map(|index| prepared.remove(index))
    };
    match prepared {
        Some(prepared) if prepared.source == source => prepared.text,
        _ => DecodedText::owned(bytes),
    }
}

pub(in crate::editor) fn discard_decoded(path: &Path) {
    lock().retain(|entry| entry.path != path);
}

fn lock() -> MutexGuard<'static, Vec<Prepared>> {
    PREPARED.lock().unwrap_or_else(PoisonError::into_inner)
}
