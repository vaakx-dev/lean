mod choices;
mod default_shell;
mod environment_field;
mod page;

use std::collections::BTreeMap;

use crate::settings::Setting;

pub(crate) use choices::{BellMode, ClipboardAccess, CursorShapeChoice, WorkingDirectory};
pub(crate) use default_shell::default_shell_program;
pub(crate) use page::settings_page;

pub(crate) const SHELL_PROGRAM: Setting<String> =
    Setting::new("terminal.shell.program", String::new());
pub(crate) const SHELL_ARGS: Setting<Vec<String>> = Setting::new("terminal.shell.args", Vec::new());
pub(crate) const SHELL_LOGIN: Setting<bool> = Setting::new("terminal.shell.login", false);
pub(crate) const ENVIRONMENT: Setting<BTreeMap<String, String>> =
    Setting::new("terminal.env", BTreeMap::new());
pub(crate) const WORKING_DIRECTORY: Setting<WorkingDirectory> =
    Setting::new("terminal.working_directory", WorkingDirectory::ProjectRoot);

pub(crate) const MIN_FONT_SIZE: f32 = 6.;
pub(crate) const MAX_FONT_SIZE: f32 = 72.;
pub(crate) const MIN_LINE_HEIGHT: f32 = 1.;
pub(crate) const MAX_LINE_HEIGHT: f32 = 2.;

pub(crate) const FONT_FAMILY: Setting<Option<String>> = Setting::new("terminal.font_family", None);
pub(crate) const FONT_SIZE: Setting<Option<f32>> = Setting::new("terminal.font_size", None);
pub(crate) const LINE_HEIGHT: Setting<f32> = Setting::new("terminal.line_height", 1.3);
pub(crate) const CURSOR_SHAPE: Setting<CursorShapeChoice> =
    Setting::new("terminal.cursor_shape", CursorShapeChoice::Block);
pub(crate) const CURSOR_BLINK: Setting<bool> = Setting::new("terminal.cursor_blink", false);

pub(crate) const SCROLLBACK: Setting<u32> = Setting::new("terminal.scrollback", 10_000);
pub(crate) const COPY_ON_SELECT: Setting<bool> = Setting::new("terminal.copy_on_select", false);
pub(crate) const BELL: Setting<BellMode> = Setting::new("terminal.bell", BellMode::None);
pub(crate) const OSC52: Setting<ClipboardAccess> =
    Setting::new("terminal.osc52", ClipboardAccess::Copy);
pub(crate) const CONFIRM_CLOSE_RUNNING: Setting<bool> =
    Setting::new("terminal.confirm_close_running", true);

pub(crate) const PANEL_HEIGHT: Setting<f32> = Setting::new("workbench.terminal.height", 260.);

pub(crate) const PASSTHROUGH_KEYS: Setting<Vec<String>> =
    Setting::new_with("terminal.passthrough_keys", default_passthrough_keys);

fn default_passthrough_keys() -> Vec<String> {
    [
        "ctrl-c",
        "ctrl-r",
        "ctrl-w",
        "ctrl-b",
        "ctrl-s",
        "ctrl-o",
        "tab",
        "shift-tab",
    ]
    .map(String::from)
    .to_vec()
}
