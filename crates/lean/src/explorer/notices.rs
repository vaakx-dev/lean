use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, SharedString};

use super::Explorer;

impl Explorer {
    pub(super) fn show_warning(&self, message: impl Into<SharedString>, cx: &mut App) {
        let window = self.window;
        let message = message.into();
        cx.defer(move |cx| {
            let shown = window.update(cx, |_, window, cx| {
                window.push_notification(Notification::warning(message), cx)
            });
            if let Err(error) = shown {
                log::debug!("failed to show an explorer warning: {error:#}");
            }
        });
    }
}
