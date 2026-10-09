use std::io;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use tokio::runtime::{Builder, Runtime};

static RUNTIME: LazyLock<io::Result<Runtime>> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("lean-kernels")
        .enable_all()
        .build()
});

pub(super) fn runtime() -> Result<&'static Runtime> {
    RUNTIME
        .as_ref()
        .map_err(|error| anyhow!("failed to start the kernel runtime: {error}"))
}
