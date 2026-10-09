use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::{App, Entity, Window};

use super::file_view::AnyFileView;
use crate::toolchain::Toolchains;

#[derive(Clone, Copy)]
pub(crate) struct FileViewer {
    pub(crate) priority: i32,
    pub(crate) handles: fn(&Path) -> bool,
    pub(crate) prepare: Option<fn(&Path, &[u8])>,
    pub(crate) open: fn(OpenRequest, &mut Window, &mut App) -> Result<AnyFileView>,
}

pub(crate) struct OpenRequest {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
    pub(crate) toolchains: Entity<Toolchains>,
}
