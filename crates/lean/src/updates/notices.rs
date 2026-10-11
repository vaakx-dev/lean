use gpui_kit::App;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Sizable as _, WindowExt as _};

use super::github::Release;
use super::{Installed, start_install};

struct UpdateNotice;

pub(super) fn show(notification: Notification, cx: &mut App) {
    let Some(window) = cx.active_window().or_else(|| cx.windows().first().copied()) else {
        return;
    };
    window
        .update(cx, |_, window, cx| {
            window.push_notification(notification.id::<UpdateNotice>(), cx)
        })
        .ok();
}

pub(super) fn available(release: Release) -> Notification {
    Notification::info(format!("Lean {} is available.", release.label())).action(move |_, _, _| {
        let release = release.clone();
        Button::new("install-update")
            .primary()
            .small()
            .label("Install")
            .on_click(move |_, _, cx| start_install(release.clone(), cx))
    })
}

pub(super) fn downloading(label: &str) -> Notification {
    Notification::info(format!("Downloading Lean {label}…")).autohide(false)
}

pub(super) fn installed(installed: Installed) -> Notification {
    Notification::success(format!(
        "Lean {} is installed. Restart to use it.",
        installed.label
    ))
    .action(move |_, _, _| {
        let program = installed.program.clone();
        Button::new("restart-for-update")
            .primary()
            .small()
            .label("Restart")
            .on_click(move |_, _, cx| crate::app::restart_as(program.clone(), cx))
    })
}

pub(super) fn up_to_date() -> Notification {
    Notification::info("Lean is up to date.")
}

pub(super) fn not_updatable() -> Notification {
    Notification::info(
        "This copy of Lean wasn't installed from a GitHub release, so it can't update itself.",
    )
}

pub(super) fn failed(error: &anyhow::Error) -> Notification {
    Notification::error(format!("Couldn't update Lean: {error:#}"))
}
