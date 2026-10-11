use std::path::Path;

use gpui_kit::*;

use crate::editor::{EditorArea, EditorAreaEvent};
use crate::files::settings::HOT_EXIT;

impl EditorArea {
    pub(crate) fn start_saving_services(
        &mut self,
        project_root: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        self.start_hot_exit(project_root, window, cx);
        cx.on_release(|area, cx| area.back_up_unsaved(true, cx))
            .detach();
        let mut active = self.active_file().map(|file| file.id);
        let area = cx.entity();
        let own_events = cx.subscribe_in(
            &area,
            window,
            move |area, _, event: &EditorAreaEvent, window, cx| {
                match event {
                    EditorAreaEvent::FileEdited(path) => area.schedule_auto_save(path, window, cx),
                    EditorAreaEvent::ActiveEditorChanged => {
                        let current = area.active_file().map(|file| file.id);
                        if let Some(previous) = active.filter(|previous| Some(*previous) != current)
                        {
                            area.auto_save_on_focus_change(previous, window, cx);
                        }
                        active = current;
                    }
                    _ => {}
                }
                area.schedule_backup(window, cx);
            },
        );
        [
            HOT_EXIT.observe_in(window, cx, |area, _, window, cx| {
                area.schedule_backup(window, cx)
            }),
            cx.on_app_quit(|area, cx| {
                area.back_up_unsaved(true, cx);
                async {}
            }),
            cx.on_focus_out(&self.focus_handle, window, |area, _, window, cx| {
                if let Some(id) = area.active_file().map(|file| file.id) {
                    area.auto_save_on_focus_change(id, window, cx);
                }
            }),
            cx.observe_window_activation(window, |area, window, cx| {
                if !window.is_window_active() {
                    area.auto_save_all_on_focus_change(window, cx);
                }
            }),
        ]
        .into_iter()
        .fold(own_events, Subscription::join)
    }
}
