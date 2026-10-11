use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::{Icon, Sizable as _};
use gpui_kit::*;

use crate::editor::EditorArea;
use crate::settings::global::snapshot;
use crate::settings::settings_file;
use crate::storage::write_json_atomic;

pub(super) fn edit_json_button(editor_area: WeakEntity<EditorArea>) -> Button {
    Button::new("edit-settings-json")
        .outline()
        .small()
        .icon(Icon::empty().path("icons/braces.svg"))
        .label("Edit settings.json")
        .on_click(move |_, window, cx| open_settings_json(&editor_area, window, cx))
}

fn open_settings_json(editor_area: &WeakEntity<EditorArea>, window: &mut Window, cx: &mut App) {
    let Some(file) = settings_file() else {
        window.push_notification("Lean has no config folder to keep settings.json in.", cx);
        return;
    };
    if !file.exists()
        && let Err(error) = write_json_atomic(&file, &snapshot(cx))
    {
        log::warn!("failed to create {}: {error:#}", file.display());
        window.push_notification(format!("Couldn't create settings.json: {error:#}"), cx);
        return;
    }
    editor_area
        .update(cx, |area, cx| area.open(file, window, cx))
        .ok();
}
