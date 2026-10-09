use std::collections::VecDeque;
use std::io;
use std::process::ExitStatus;

use anyhow::{Error, anyhow};
use tokio::io::{AsyncBufReadExt as _, BufReader, Lines};
use tokio::process::ChildStderr;

const TAIL_LINES: usize = 12;

pub(super) struct KernelStderr {
    lines: Lines<BufReader<ChildStderr>>,
    tail: VecDeque<String>,
    open: bool,
}

impl KernelStderr {
    pub(super) fn new(stderr: ChildStderr) -> Self {
        Self {
            lines: BufReader::new(stderr).lines(),
            tail: VecDeque::new(),
            open: true,
        }
    }

    pub(super) fn is_open(&self) -> bool {
        self.open
    }

    pub(super) async fn next_line(&mut self) -> io::Result<Option<String>> {
        self.lines.next_line().await
    }

    pub(super) fn record(&mut self, line: Option<String>) {
        let Some(line) = line else {
            self.open = false;
            return;
        };
        log::debug!("kernel: {line}");
        if self.tail.len() == TAIL_LINES {
            self.tail.pop_front();
        }
        self.tail.push_back(line);
    }

    pub(super) async fn exit_error(&mut self, status: ExitStatus) -> Error {
        while self.open {
            match self.next_line().await {
                Ok(line) => self.record(line),
                Err(error) => {
                    log::debug!("failed to read the kernel output: {error}");
                    break;
                }
            }
        }
        let output = self.tail.iter().cloned().collect::<Vec<_>>().join("\n");
        if output.is_empty() {
            anyhow!("the kernel exited ({status})")
        } else {
            anyhow!("the kernel exited ({status}): {output}")
        }
    }
}
