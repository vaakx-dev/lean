use gpui_kit::*;
use serde::{Deserialize, Serialize};

use super::read_state;
use crate::app::settings::RESTORE_WINDOW_BOUNDS;
use crate::storage::{QueuedWriter, state_file};
use crate::workspace::Workspace;
use crate::workspace::settings::RESTORE_LAYOUT;
use crate::workspace::sidebar_resize::clamp_sidebar_width;

const FILE_NAME: &str = "window-state.json";

static WRITER: QueuedWriter = QueuedWriter::new();

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct WindowState {
    bounds: Option<SavedBounds>,
    layout: Option<SavedLayout>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SavedBounds {
    mode: WindowMode,
    bounds: Bounds<Pixels>,
}

#[derive(Serialize, Deserialize)]
struct SavedLayout {
    sidebar_visible: bool,
    panel: String,
    sidebar_width: f32,
    terminal_visible: bool,
}

pub(crate) fn restored_window_bounds(min_size: Size<Pixels>, cx: &App) -> Option<WindowBounds> {
    if !RESTORE_WINDOW_BOUNDS.get(cx) {
        return None;
    }
    let saved = read_state::<WindowState>(FILE_NAME)?.bounds?;
    let bounds = saved.bounds;
    let fits = bounds.size.width >= min_size.width && bounds.size.height >= min_size.height;
    let on_screen = cx
        .displays()
        .iter()
        .any(|display| display.bounds().intersects(&bounds));
    if !fits || !on_screen {
        return None;
    }
    Some(match saved.mode {
        WindowMode::Windowed => WindowBounds::Windowed(bounds),
        WindowMode::Maximized => WindowBounds::Maximized(bounds),
        WindowMode::Fullscreen => WindowBounds::Fullscreen(bounds),
    })
}

impl Workspace {
    pub(crate) fn save_window_state(&self, window: &Window) {
        let Some(file) = state_file(FILE_NAME) else {
            return;
        };
        let state = WindowState {
            bounds: Some(saved_bounds(window.window_bounds())),
            layout: Some(self.saved_layout()),
        };
        WRITER.save_now(&file, &state);
    }

    pub(in crate::workspace) fn restore_layout(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !RESTORE_LAYOUT.get(cx) {
            return;
        }
        let Some(layout) = read_state::<WindowState>(FILE_NAME).and_then(|state| state.layout)
        else {
            return;
        };
        if let Some(index) = self.panel_index(&layout.panel) {
            self.active_panel = index;
        }
        self.sidebar_visible = layout.sidebar_visible;
        self.sidebar_width = clamp_sidebar_width(layout.sidebar_width);
        if layout.terminal_visible {
            self.terminal_visible = true;
            self.reset_terminal(window, cx);
        }
    }

    fn saved_layout(&self) -> SavedLayout {
        let panel = self
            .panels
            .get(self.active_panel)
            .map(|panel| panel.panel.id.to_owned())
            .unwrap_or_default();
        SavedLayout {
            sidebar_visible: self.sidebar_visible,
            panel,
            sidebar_width: self.sidebar_width.as_f32(),
            terminal_visible: self.terminal_visible,
        }
    }
}

fn saved_bounds(window_bounds: WindowBounds) -> SavedBounds {
    let (mode, bounds) = match window_bounds {
        WindowBounds::Windowed(bounds) => (WindowMode::Windowed, bounds),
        WindowBounds::Maximized(bounds) => (WindowMode::Maximized, bounds),
        WindowBounds::Fullscreen(bounds) => (WindowMode::Fullscreen, bounds),
    };
    SavedBounds { mode, bounds }
}
