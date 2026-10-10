use gpui_kit::{Context, Subscription};

use super::SourceControl;
use crate::plugins::git::settings::{COMPACT_FOLDERS, SHOW_HISTORY, VIEW_MODE};

impl SourceControl {
    pub(super) fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            VIEW_MODE.observe(cx, |source_control, view_mode, cx| {
                source_control.set_view_mode(view_mode, cx)
            }),
            COMPACT_FOLDERS.observe(cx, |source_control, compact, cx| {
                source_control.compact_folders = compact;
                source_control.rebuild_rows();
                cx.notify();
            }),
            SHOW_HISTORY.observe(cx, |source_control, show, cx| {
                source_control.show_history = show;
                source_control.rebuild_rows();
                cx.notify();
            }),
        ]
    }
}
