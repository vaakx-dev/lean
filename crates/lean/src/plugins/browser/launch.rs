use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

pub(super) fn launch(command: &str, path: &Path) -> io::Result<()> {
    let mut child = Command::new(command)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::spawn(move || {
        if let Err(error) = child.wait() {
            log::debug!("failed to wait for the browser: {error}");
        }
    });
    Ok(())
}
