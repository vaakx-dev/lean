use std::path::{Path, PathBuf};
use std::process::Command;

use gpui_kit::component::Root;
use gpui_kit::*;

use crate::settings;
use crate::workspace::{CloseIntent, Workspace};

pub(super) fn quit(cx: &mut App) {
    close_windows_then(cx, |cx| cx.quit());
}

pub(crate) fn restart(cx: &mut App) {
    match std::env::current_exe() {
        Ok(program) => restart_as(program, cx),
        Err(error) => log::error!("failed to restart Lean: {error}"),
    }
}

pub(crate) fn restart_as(program: PathBuf, cx: &mut App) {
    close_windows_then(cx, move |cx| {
        settings::save_now();
        match relaunch(&program) {
            Ok(()) => cx.quit(),
            Err(error) => log::error!("failed to restart Lean: {error}"),
        }
    });
}

fn close_windows_then(cx: &mut App, then: impl FnOnce(&mut App) + 'static) {
    cx.spawn(async move |cx| {
        let windows = cx.update(|cx| cx.windows());
        for window in windows {
            if !confirm_quit(window, cx).await {
                return;
            }
        }
        cx.update(then);
    })
    .detach();
}

fn relaunch(program: &Path) -> std::io::Result<()> {
    Command::new(program)
        .args(std::env::args_os().skip(1))
        .spawn()?;
    Ok(())
}

async fn confirm_quit(window: AnyWindowHandle, cx: &mut AsyncApp) -> bool {
    let confirmation = window.update(cx, |root, window, cx| {
        let workspace = workspace_of(root, cx)?;
        Some(workspace.update(cx, |workspace, cx| {
            workspace.save_window_state(window);
            workspace.confirm_close(CloseIntent::Quit, window, cx)
        }))
    });
    match confirmation.ok().flatten() {
        Some(confirmation) => confirmation.await,
        None => true,
    }
}

fn workspace_of(root: AnyView, cx: &App) -> Option<Entity<Workspace>> {
    let root = root.downcast::<Root>().ok()?;
    root.read(cx).view().clone().downcast::<Workspace>().ok()
}
