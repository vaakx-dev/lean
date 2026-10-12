#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";
#[cfg(windows)]
const WINDOWS_SHELL: &str = "powershell.exe";

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
