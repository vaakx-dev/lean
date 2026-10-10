use std::path::Path;

use gpui_kit::*;

use super::hot_exit::{HotExit, SharedHotExit};
use crate::editor::{EditorArea, EditorAreaEvent};
use crate::files::settings::HOT_EXIT;

impl EditorArea {
    pub(crate) fn start_saving_services(
        &mut self,
        project_root: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        let hot_exit = HotExit::start(project_root, window, cx);
        let release = hot_exit.clone();
        cx.on_release(move |area, cx| release.borrow_mut().write(area, cx, true))
            .detach();
        [
            self.observe_own_events(hot_exit.clone(), window, cx),
            self.observe_hot_exit_setting(hot_exit.clone(), window, cx),
            cx.on_app_quit(move |area, cx| {
                hot_exit.borrow_mut().write(area, cx, true);
                async {}
            }),
            cx.on_focus_out(&self.focus_handle, window, |area, _, window, cx| {
                area.auto_save_active_on_focus_change(window, cx)
            }),
            cx.observe_window_activation(window, |area, window, cx| {
                if !window.is_window_active() {
                    area.auto_save_all_on_focus_change(window, cx);
                }
            }),
        ]
        .into_iter()
        .reduce(Subscription::join)
        .unwrap_or_else(|| Subscription::new(|| {}))
    }

    fn observe_own_events(
        &self,
        hot_exit: SharedHotExit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        let mut active = self.active_file().map(|file| file.id);
        let area = cx.entity();
        cx.subscribe_in(
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
                HotExit::schedule(&hot_exit, area, window, cx);
            },
        )
    }

    fn observe_hot_exit_setting(
        &self,
        hot_exit: SharedHotExit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Subscription {
        HOT_EXIT.observe_in(window, cx, move |area, _, window, cx| {
            HotExit::schedule(&hot_exit, area, window, cx)
        })
    }
}
