use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{Context, Subscription, Window};

use super::Explorer;
use crate::editor::EditorAreaEvent;
use crate::plugins::WorkspaceContext;
use crate::project::{FilesChanged, RevealRequested, WatchFailed};
use crate::settings::Settings;

pub(super) fn subscribe(
    workspace: &WorkspaceContext,
    window: &Window,
    cx: &mut Context<Explorer>,
) -> Vec<Subscription> {
    let project = &workspace.project;
    vec![
        cx.observe(&workspace.decorations, |_, _, cx| cx.notify()),
        cx.observe_global::<Settings>(|explorer, cx| explorer.apply_options(cx)),
        cx.subscribe_in(
            project,
            window,
            |explorer, _, event: &FilesChanged, window, cx| {
                explorer.apply_fs_changes(&event.0, window, cx)
            },
        ),
        cx.subscribe(project, |explorer, _, event: &RevealRequested, cx| {
            explorer.reveal_path(event.0.clone(), cx)
        }),
        cx.subscribe_in(project, window, |_, _, event: &WatchFailed, window, cx| {
            let notification = Notification::warning(event.0.clone()).autohide(false);
            window.push_notification(notification, cx);
        }),
        cx.subscribe(
            &workspace.editor_area,
            |explorer, _, event: &EditorAreaEvent, cx| match event {
                EditorAreaEvent::FileRecreated(path) => explorer.show_recreated_file(path, cx),
                EditorAreaEvent::ActiveEditorChanged => explorer.follow_active_editor(cx),
                _ => {}
            },
        ),
    ]
}
