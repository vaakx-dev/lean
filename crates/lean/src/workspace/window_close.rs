use gpui_kit::*;

use super::Workspace;
use super::close_confirmation::CloseIntent;

impl Workspace {
    pub(super) fn guard_window_close(window: &Window, cx: &Context<Self>) {
        let workspace = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            may_close_window(&workspace, window, cx)
        });
    }

    pub(super) fn close_window_handler(
        cx: &Context<Self>,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let workspace = cx.weak_entity();
        move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
            if may_close_window(&workspace, window, cx) {
                window.remove_window();
            }
        }
    }

    fn allows_window_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.close_confirmed || !self.needs_close_confirmation(CloseIntent::Quit, cx) {
            return true;
        }
        let confirmation = self.confirm_close(CloseIntent::Quit, window, cx);
        cx.spawn_in(window, async move |workspace, cx| {
            if !confirmation.await {
                return;
            }
            let update = workspace.update_in(cx, |workspace, window, _| {
                workspace.close_confirmed = true;
                window.remove_window();
            });
            if let Err(error) = update {
                log::debug!("workspace dropped while closing its window: {error:#}");
            }
        })
        .detach();
        false
    }
}

fn may_close_window(workspace: &WeakEntity<Workspace>, window: &mut Window, cx: &mut App) -> bool {
    workspace
        .update(cx, |workspace, cx| {
            workspace.allows_window_close(window, cx)
        })
        .unwrap_or(true)
}
