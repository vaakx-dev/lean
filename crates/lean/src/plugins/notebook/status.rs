use gpui_kit::SharedString;

use super::Notebook;

pub(crate) enum KernelStatus {
    Stopped,
    Starting,
    Idle,
    Busy,
    Failed(SharedString),
}

impl KernelStatus {
    pub(crate) fn label(&self) -> SharedString {
        match self {
            KernelStatus::Stopped => "Kernel stopped".into(),
            KernelStatus::Starting => "Starting kernel…".into(),
            KernelStatus::Idle => "Kernel idle".into(),
            KernelStatus::Busy => "Kernel busy".into(),
            KernelStatus::Failed(_) => "Kernel failed".into(),
        }
    }
}

impl Notebook {
    pub(crate) fn kernel_status(&self) -> KernelStatus {
        if let Some(error) = &self.kernel_error {
            return KernelStatus::Failed(error.clone());
        }
        match &self.session {
            None if self.awaiting_toolchains => KernelStatus::Starting,
            None => KernelStatus::Stopped,
            Some(session) if !session.ready => KernelStatus::Starting,
            Some(session) if session.running.is_some() => KernelStatus::Busy,
            Some(_) => KernelStatus::Idle,
        }
    }

    pub(crate) fn selection_label(&self) -> SharedString {
        match self.selected.and_then(|id| self.index_of(id)) {
            Some(index) => format!("Cell {} of {}", index + 1, self.cells.len()).into(),
            None => format!("{} cells", self.cells.len()).into(),
        }
    }
}
