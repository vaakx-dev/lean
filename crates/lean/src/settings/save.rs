use std::time::Duration;

use gpui_kit::{App, Task};

use super::global::{Settings, snapshot};
use super::settings_file;
use crate::storage::QueuedWriter;

const SAVE_DELAY: Duration = Duration::from_millis(300);

static WRITER: QueuedWriter = QueuedWriter::new();

pub(super) fn schedule(cx: &App) -> Task<()> {
    cx.spawn(async move |cx| {
        cx.background_executor().timer(SAVE_DELAY).await;
        cx.update(flush);
    })
}

pub(super) fn is_busy(cx: &App) -> bool {
    let pending = cx
        .try_global::<Settings>()
        .is_some_and(|settings| settings.save_pending.get());
    pending || !WRITER.is_idle()
}

pub(super) fn flush_now(cx: &App) {
    let pending = take_pending(cx);
    if (pending || !WRITER.is_idle())
        && let Some(file) = settings_file()
    {
        WRITER.save_now(&file, &snapshot(cx));
    }
}

fn flush(cx: &mut App) {
    if take_pending(cx)
        && let Some(file) = settings_file()
    {
        WRITER.save(file, snapshot(cx), cx);
    }
}

fn take_pending(cx: &App) -> bool {
    cx.try_global::<Settings>()
        .is_some_and(|settings| settings.save_pending.replace(false))
}
