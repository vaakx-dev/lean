use alacritty_terminal::tty::Pty;

#[cfg(unix)]
pub(super) struct ForegroundProbe {
    shell_pid: i32,
    terminal: std::fs::File,
}

#[cfg(unix)]
impl ForegroundProbe {
    pub(super) fn new(pty: &Pty) -> Option<Self> {
        Some(Self {
            shell_pid: i32::try_from(pty.child().id()).ok()?,
            terminal: pty.file().try_clone().ok()?,
        })
    }

    pub(super) fn runs_job(&self) -> bool {
        use std::os::fd::AsRawFd as _;
        let group = tcgetpgrp(self.terminal.as_raw_fd());
        group > 0 && group != self.shell_pid
    }
}

#[cfg(unix)]
unsafe extern "C" {
    safe fn tcgetpgrp(fd: std::ffi::c_int) -> i32;
}

#[cfg(not(unix))]
pub(super) enum ForegroundProbe {}

#[cfg(not(unix))]
impl ForegroundProbe {
    pub(super) fn new(_: &Pty) -> Option<Self> {
        None
    }

    pub(super) fn runs_job(&self) -> bool {
        match *self {}
    }
}
