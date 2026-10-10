use std::sync::atomic::{AtomicU32, Ordering};

use gpui_kit::App;

use super::{DEFAULT_HIGHLIGHT_LIMIT_MB, DEFAULT_MAX_SIZE_MB, HIGHLIGHT_LIMIT_MB, MAX_SIZE_MB};

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

static MAX_SIZE: AtomicU32 = AtomicU32::new(DEFAULT_MAX_SIZE_MB);
static HIGHLIGHT_LIMIT: AtomicU32 = AtomicU32::new(DEFAULT_HIGHLIGHT_LIMIT_MB);

pub(crate) fn init(cx: &mut App) {
    MAX_SIZE.store(MAX_SIZE_MB.get(cx), Ordering::Relaxed);
    HIGHLIGHT_LIMIT.store(HIGHLIGHT_LIMIT_MB.get(cx), Ordering::Relaxed);
    MAX_SIZE_MB
        .observe_app(cx, |megabytes, _| {
            MAX_SIZE.store(megabytes, Ordering::Relaxed)
        })
        .detach();
    HIGHLIGHT_LIMIT_MB
        .observe_app(cx, |megabytes, _| {
            HIGHLIGHT_LIMIT.store(megabytes, Ordering::Relaxed)
        })
        .detach();
}

pub(crate) fn max_file_bytes() -> u64 {
    megabytes(MAX_SIZE.load(Ordering::Relaxed).clamp(1, 2048))
}

pub(crate) fn highlight_limit_bytes() -> usize {
    let limit = megabytes(HIGHLIGHT_LIMIT.load(Ordering::Relaxed).clamp(1, 64));
    usize::try_from(limit).unwrap_or(usize::MAX)
}

fn megabytes(count: u32) -> u64 {
    u64::from(count) * BYTES_PER_MEGABYTE
}
