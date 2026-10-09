use gpui_kit::{AnyEntity, App, Window};

use super::workspace_context::WorkspaceContext;

pub(crate) type StartService = fn(&WorkspaceContext, &mut Window, &mut App) -> AnyEntity;
