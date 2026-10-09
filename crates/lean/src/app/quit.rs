use gpui_kit::component::Root;
use gpui_kit::*;

use crate::workspace::{CloseIntent, Workspace};

pub(super) fn quit(cx: &mut App) {
    cx.spawn(async move |cx| {
        let windows = cx.update(|cx| cx.windows());
        for window in windows {
            if !confirm_quit(window, cx).await {
                return;
            }
        }
        cx.update(|cx| cx.quit());
    })
    .detach();
}

async fn confirm_quit(window: AnyWindowHandle, cx: &mut AsyncApp) -> bool {
    let confirmation = window.update(cx, |root, window, cx| {
        let workspace = workspace_of(root, cx)?;
        Some(workspace.update(cx, |workspace, cx| {
            workspace.confirm_close(CloseIntent::Quit, window, cx)
        }))
    });
    match confirmation {
        Ok(Some(confirmation)) => confirmation.await,
        Ok(None) => true,
        Err(error) => {
            log::debug!("a window closed while quitting: {error:#}");
            true
        }
    }
}

fn workspace_of(root: AnyView, cx: &App) -> Option<Entity<Workspace>> {
    let root = root.downcast::<Root>().ok()?;
    root.read(cx).view().clone().downcast::<Workspace>().ok()
}
