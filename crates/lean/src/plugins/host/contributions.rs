use std::path::{Path, PathBuf};

use anyhow::Result;
use gpui_kit::{AnyEntity, AnyView, App, Entity, Window};

use super::file_view::AnyFileView;
use super::workspace_context::WorkspaceContext;
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

#[derive(Clone, Copy)]
pub(crate) struct FileAction {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) applies: fn(&Path, &App) -> bool,
    pub(crate) run: fn(&Path, &mut Window, &mut App),
}

#[derive(Clone, Copy)]
pub(crate) struct StatusItem {
    pub(crate) order: i32,
    pub(crate) build: fn(&WorkspaceContext, &mut Window, &mut App) -> AnyView,
}

pub(crate) type StartService = fn(&WorkspaceContext, &mut Window, &mut App) -> AnyEntity;
