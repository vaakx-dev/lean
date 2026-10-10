use alacritty_terminal::tty::Pty;

pub(super) struct ShellProcess {
    pid: Option<u32>,
}

impl ShellProcess {
    #[cfg(unix)]
    pub(super) fn of(pty: &Pty) -> Self {
        Self {
            pid: Some(pty.child().id()),
        }
    }

    #[cfg(not(unix))]
    pub(super) fn of(_: &Pty) -> Self {
        Self { pid: None }
    }

    pub(super) fn has_children(&self) -> bool {
        self.pid.is_none_or(has_child_processes)
    }
}

#[cfg(target_os = "linux")]
fn has_child_processes(pid: u32) -> bool {
    match std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children")) {
        Ok(children) => !children.trim().is_empty(),
        Err(_) => true,
    }
}

#[cfg(not(target_os = "linux"))]
fn has_child_processes(_: u32) -> bool {
    true
}
