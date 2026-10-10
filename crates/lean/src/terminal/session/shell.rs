use std::collections::HashMap;
use std::path::Path;

use alacritty_terminal::tty::{Options, Shell};
use gpui_kit::{App, SharedString};

use crate::terminal::settings::{ENVIRONMENT, SHELL_ARGS, SHELL_LOGIN, SHELL_PROGRAM};

#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";
#[cfg(windows)]
const WINDOWS_SHELL: &str = "powershell.exe";
const LOGIN_FLAG: &str = "-l";

pub(crate) fn shell_options(
    working_directory: &Path,
    environment: &HashMap<String, String>,
    cx: &App,
) -> Options {
    let mut env = shell_environment();
    env.extend(environment.clone());
    env.extend(ENVIRONMENT.get(cx));
    Options {
        shell: Some(Shell::new(shell_program(cx), shell_args(cx))),
        working_directory: Some(working_directory.to_path_buf()),
        drain_on_exit: true,
        env,
        ..Options::default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellKind {
    Posix,
    Fish,
    PowerShell,
    Other,
}

pub(crate) fn shell_kind(cx: &App) -> ShellKind {
    let name = shell_name(cx).to_lowercase();
    match name.strip_suffix(".exe").unwrap_or(&name) {
        "sh" | "bash" | "zsh" | "dash" | "ksh" => ShellKind::Posix,
        "fish" => ShellKind::Fish,
        "pwsh" | "powershell" => ShellKind::PowerShell,
        _ => ShellKind::Other,
    }
}

pub(crate) fn shell_name(cx: &App) -> SharedString {
    let program = shell_program(cx);
    Path::new(&program)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or(program)
        .into()
}

fn shell_program(cx: &App) -> String {
    let configured = SHELL_PROGRAM.get(cx);
    let configured = configured.trim();
    if configured.is_empty() {
        default_shell_program()
    } else {
        configured.to_owned()
    }
}

fn shell_args(cx: &App) -> Vec<String> {
    let login = (cfg!(not(windows)) && SHELL_LOGIN.get(cx)).then(|| LOGIN_FLAG.to_owned());
    login.into_iter().chain(SHELL_ARGS.get(cx)).collect()
}

#[cfg(windows)]
pub(crate) fn default_shell_program() -> String {
    WINDOWS_SHELL.to_owned()
}

#[cfg(not(windows))]
pub(crate) fn default_shell_program() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|shell| !shell.is_empty())
        .unwrap_or_else(|| FALLBACK_SHELL.to_owned())
}

fn shell_environment() -> HashMap<String, String> {
    HashMap::from([
        ("TERM".to_owned(), "xterm-256color".to_owned()),
        ("COLORTERM".to_owned(), "truecolor".to_owned()),
        ("TERM_PROGRAM".to_owned(), "lean".to_owned()),
    ])
}
