use super::Toolchain;
use crate::terminal::ShellKind;

pub(crate) struct ShellActivation {
    pub(crate) posix: String,
    pub(crate) fish: String,
    pub(crate) powershell: String,
}

impl ShellActivation {
    fn command(&self, shell: ShellKind) -> Option<&str> {
        match shell {
            ShellKind::Posix => Some(self.posix.as_str()),
            ShellKind::Fish => Some(self.fish.as_str()),
            ShellKind::PowerShell => Some(self.powershell.as_str()),
            ShellKind::Other => None,
        }
    }
}

pub(super) fn activation_command(active: &[&Toolchain], shell: ShellKind) -> Option<String> {
    let commands: Vec<&str> = active
        .iter()
        .filter_map(|toolchain| toolchain.activation.as_ref()?.command(shell))
        .collect();
    (!commands.is_empty()).then(|| format!(" {}", commands.join("; ")))
}

pub(super) fn posix_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(super) fn fish_quote(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

pub(super) fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
