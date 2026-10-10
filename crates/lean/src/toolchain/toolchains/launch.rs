use std::collections::HashMap;

use gpui_kit::App;

use super::Toolchains;
use crate::terminal::{ShellKind, TerminalLaunch};
use crate::toolchain::activation::{activation_command, activation_environment};
use crate::toolchain::settings::{ACTIVATE_IN_TERMINAL, ACTIVATION_MODE, ActivationMode};

impl Toolchains {
    pub(crate) fn environment(&self) -> HashMap<String, String> {
        activation_environment(&self.available, &self.active_toolchains())
    }

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
        }
    }
}
