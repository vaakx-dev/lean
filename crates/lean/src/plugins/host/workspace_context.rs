use std::rc::Rc;

use gpui_kit::{AnyEntity, App, Entity, Window};

use super::file_decorations::FileDecorations;
use super::registry::services;
use crate::editor::EditorArea;
use crate::project::Project;
use crate::toolchain::Toolchains;

#[derive(Clone)]
pub(crate) struct WorkspaceContext {
    pub(crate) project: Entity<Project>,
    pub(crate) toolchains: Entity<Toolchains>,
    pub(crate) editor_area: Entity<EditorArea>,
    pub(crate) decorations: Entity<FileDecorations>,
    services: Rc<[AnyEntity]>,
}

impl WorkspaceContext {
    pub(crate) fn new(
        project: Entity<Project>,
        toolchains: Entity<Toolchains>,
        editor_area: Entity<EditorArea>,
        decorations: Entity<FileDecorations>,
    ) -> Self {
        Self {
            project,
            toolchains,
            editor_area,
            decorations,
            services: Rc::new([]),
        }
    }

    pub(crate) fn start_services(mut self, window: &mut Window, cx: &mut App) -> Self {
        let started: Vec<AnyEntity> = services(cx)
            .into_iter()
            .map(|start| start(&self, window, cx))
            .collect();
        self.services = started.into();
        self
    }

    pub(crate) fn service<T: 'static>(&self) -> Option<Entity<T>> {
        self.services
            .iter()
            .find_map(|service| service.clone().downcast::<T>().ok())
    }
}
