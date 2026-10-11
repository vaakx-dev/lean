use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{Context, Subscription, Window};

use super::Explorer;
use crate::editor::EditorAreaEvent;
use crate::plugins::WorkspaceContext;
use crate::project::{FilesChanged, ProjectEvent};
use crate::settings::Settings;

pub(super) fn subscribe(
    workspace: &WorkspaceContext,
    window: &Window,
    cx: &mut Context<Explorer>,
) -> Vec<Subscription> {
    let project = &workspace.project;
    vec![
        cx.observe(&workspace.decorations, |_, _, cx| cx.notify()),
        cx.observe_global_in::<Settings>(window, |explorer, window, cx| {
            explorer.apply_options(window, cx)
        }),
        cx.subscribe_in(
            project,
            window,
            |explorer, _, event: &FilesChanged, window, cx| {
                explorer.apply_fs_changes(&event.0, window, cx)
            },
        ),
        cx.subscribe_in(
            project,
            window,
            |explorer, _, event: &ProjectEvent, window, cx| match event {
                ProjectEvent::RevealRequested(path) => {
                    explorer.reveal_path(path.clone(), window, cx)
                }
                ProjectEvent::WatchFailed(message) => {
                    let notification = Notification::warning(message.clone()).autohide(false);
                    window.push_notification(notification, cx);
                }
                ProjectEvent::Activated | ProjectEvent::FilesTouched => {}
            },
        ),
        cx.subscribe_in(
            &workspace.editor_area,
            window,
            |explorer, _, event: &EditorAreaEvent, window, cx| match event {
                EditorAreaEvent::FileRecreated(path) => {
                    explorer.show_recreated_file(path, window, cx)
                }
                EditorAreaEvent::ActiveEditorChanged => explorer.follow_active_editor(window, cx),
                _ => {}
            },
        ),
    ]
}
