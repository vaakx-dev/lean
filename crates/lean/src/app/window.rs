use std::path::PathBuf;

use anyhow::Result;
use gpui_kit::component::TitleBar;
use gpui_kit::*;

use crate::workspace::Workspace;

pub(super) fn open_main_window(project_root: PathBuf, cx: &mut App) -> Result<()> {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            window_size(cx),
            cx,
        ))),
        window_min_size: Some(size(px(640.), px(400.))),
        window_decorations: Some(WindowDecorations::Client),
        app_id: Some("lean".into()),
        ..TitleBar::window_options()
    };
    gpui_kit::open_window(options, cx, |window, cx| {
        let workspace = cx.new(|cx| Workspace::new(project_root, window, cx));
        let startup_workspace = workspace.clone();
        window.defer(cx, move |window, cx| {
            startup_workspace.update(cx, |workspace, cx| workspace.focus_sidebar(window, cx))
        });
        workspace
    })?;
    Ok(())
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
