use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(super) struct Budget {
    remaining: AtomicUsize,
    truncated: AtomicBool,
}

impl Budget {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            remaining: AtomicUsize::new(limit),
            truncated: AtomicBool::new(false),
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.remaining.load(Ordering::Relaxed)
    }

    pub(super) fn claim(&self, wanted: usize) -> usize {
        let available = self
            .remaining
            .update(Ordering::Relaxed, Ordering::Relaxed, |left| {
                left.saturating_sub(wanted)
            });
        let claimed = available.min(wanted);
        if claimed < wanted {
            self.truncate();
        }
        claimed
    }

    pub(super) fn truncate(&self) {
        self.truncated.store(true, Ordering::Relaxed);
    }

    pub(super) fn is_truncated(&self) -> bool {
        self.truncated.load(Ordering::Relaxed)
    }
}
