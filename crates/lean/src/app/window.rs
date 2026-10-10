use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use super::settings::{TITLE_BAR, TitleBarStyle};
use crate::settings;
use crate::workspace::{Workspace, restored_window_bounds};

const MIN_WINDOW_SIZE: Size<Pixels> = size(px(640.), px(400.));

pub(super) fn open_main_window(
    project_root: PathBuf,
    file: Option<PathBuf>,
    cx: &mut App,
) -> Result<()> {
    let window_bounds = restored_window_bounds(MIN_WINDOW_SIZE, cx)
        .unwrap_or_else(|| WindowBounds::Windowed(Bounds::centered(None, window_size(cx), cx)));
    let options = WindowOptions {
        window_bounds: Some(window_bounds),
        window_min_size: Some(MIN_WINDOW_SIZE),
        app_id: Some("lean".into()),
        ..decorated_window_options(cx)
    };
    gpui_kit::open_window(options, cx, |window, cx| {
        let workspace = cx.new(|cx| Workspace::new(project_root, file, window, cx));
        let startup_workspace = workspace.clone();
        window.defer(cx, move |window, cx| {
            startup_workspace.update(cx, |workspace, cx| workspace.focus_initial(window, cx));
            settings::notify_load_error(window, cx);
        });
        workspace
    })?;
    Ok(())
}

fn decorated_window_options(cx: &App) -> WindowOptions {
    match TITLE_BAR.get(cx) {
        TitleBarStyle::Lean => WindowOptions {
            window_decorations: Some(WindowDecorations::Client),
            ..TitleBar::window_options()
        },
        TitleBarStyle::System => WindowOptions {
            window_decorations: Some(WindowDecorations::Server),
            ..WindowOptions::default()
        },
    }
}

fn window_size(cx: &App) -> Size<Pixels> {
    let mut window_size = size(px(1440.), px(900.));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.9);
        window_size.height = window_size.height.min(display_size.height * 0.9);
    }
    window_size
}
