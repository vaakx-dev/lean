use gpui_kit::{AnyView, App, Window};

use super::workspace_context::WorkspaceContext;

#[derive(Clone, Copy)]
pub(crate) struct StatusItem {
    pub(crate) order: i32,
    pub(crate) build: fn(&WorkspaceContext, &mut Window, &mut App) -> AnyView,
}
