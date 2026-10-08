use std::thread::JoinHandle;

use anyhow::Result;
use gpui_util::ResultExt;

use crate::DirectXDevices;

struct SendableDevices(DirectXDevices);

unsafe impl Send for SendableDevices {}

pub(crate) struct PendingDirectXDevices(JoinHandle<Result<SendableDevices>>);

impl PendingDirectXDevices {
    pub(crate) fn spawn() -> Option<Self> {
        std::thread::Builder::new()
            .name("DirectXDevices".to_owned())
            .spawn(|| DirectXDevices::new().map(SendableDevices))
            .log_err()
            .map(Self)
    }

    pub(crate) fn join(self) -> Result<DirectXDevices> {
        match self.0.join().ok().and_then(|result| result.log_err()) {
            Some(SendableDevices(devices)) => Ok(devices),
            None => DirectXDevices::new(),
        }
    }
}
