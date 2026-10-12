use gpui_kit::App;

use super::Toolchains;
use crate::terminal::{ShellKind, TerminalLaunch};
use crate::toolchain::settings::{ACTIVATE_IN_TERMINAL, ACTIVATION_MODE, ActivationMode};
use crate::toolchain::shell_activation::activation_command;

impl Toolchains {
    pub(crate) fn terminal_launch(&self, shell: ShellKind, cx: &App) -> TerminalLaunch {
        if !ACTIVATE_IN_TERMINAL.get(cx) {
            return TerminalLaunch::default();
        }
        let startup_command = match ACTIVATION_MODE.get(cx) {
            ActivationMode::ActivateScript => activation_command(&self.active_toolchains(), shell),
            ActivationMode::EnvironmentOnly => None,
        };
        TerminalLaunch {
            environment: self.environment(),
            startup_command,
            ..TerminalLaunch::default()
        }
    }
}
