use std::collections::HashMap;
use std::path::Path;

use alacritty_terminal::tty::{Options, Shell};
use gpui_kit::SharedString;

#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";
#[cfg(windows)]
const WINDOWS_SHELL: &str = "powershell.exe";

pub(crate) fn shell_options(working_directory: &Path) -> Options {
    Options {
        shell: Some(Shell::new(shell_program(), Vec::new())),
        working_directory: Some(working_directory.to_path_buf()),
        drain_on_exit: true,
        env: shell_environment(),
        ..Options::default()
    }
}

pub(crate) fn shell_name() -> SharedString {
    let program = shell_program();
    Path::new(&program)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or(program)
        .into()
}

#[cfg(windows)]
fn shell_program() -> String {
    WINDOWS_SHELL.to_owned()
}

#[cfg(not(windows))]
fn shell_program() -> String {
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
