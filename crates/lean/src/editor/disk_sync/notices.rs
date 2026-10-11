use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::*;

use super::super::EditorArea;

struct ExternalChangeNotice;

pub(super) fn changed_on_disk(name: &SharedString, id: u64) -> Notification {
    Notification::warning(format!("{name} changed on disk.")).id1::<ExternalChangeNotice>(id)
}

pub(super) fn reload_offer(
    area: WeakEntity<EditorArea>,
    name: &SharedString,
    id: u64,
) -> Notification {
    changed_on_disk(name, id).action(move |_, _, cx| {
        let notification = cx.entity().downgrade();
        let area = area.clone();
        Button::new("reload-file")
            .primary()
            .label("Reload")
            .on_click(move |_, window, cx| {
                area.update(cx, |area, cx| {
                    if let Some(index) = area.index_of_id(id) {
                        area.revert(index, window, cx);
                    }
                })
                .ok();
                notification
                    .update(cx, |notification, cx| notification.dismiss(window, cx))
                    .ok();
            })
    })
}

pub(in crate::editor) fn dismiss_disk_notice(id: u64, window: &mut Window, cx: &mut App) {
    window.remove_notification1::<ExternalChangeNotice>(id, cx);
}

pub(super) fn reloaded(name: &SharedString, id: u64) -> Notification {
    Notification::info(format!("Reloaded {name} because it changed on disk."))
        .id1::<ExternalChangeNotice>(id)
}
