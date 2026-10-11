use gpui_kit::*;

use super::Workspace;
use crate::plugins::{AnyPanelView, Panel, PanelEvent, WorkspaceContext, panels};

pub(super) struct SidebarPanel {
    pub(super) panel: Panel,
    pub(super) view: AnyPanelView,
    _subscription: Subscription,
}

pub(super) fn build_panels(
    context: &WorkspaceContext,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Vec<SidebarPanel> {
    let registered = panels(cx).to_vec();
    registered
        .into_iter()
        .map(|panel| {
            let view = (panel.build)(context, window, cx);
            let editor_area = context.editor_area.downgrade();
            let subscription = view.subscribe(
                window,
                cx,
                Box::new(
                    move |event: &PanelEvent, window: &mut Window, cx: &mut App| {
                        editor_area
                            .update(cx, |area, cx| match event {
                                PanelEvent::OpenFile(path) => area.open(path.clone(), window, cx),
                                PanelEvent::OpenFileAt { path, line, column } => {
                                    area.open_at(path.clone(), *line, *column, window, cx)
                                }
                                PanelEvent::PathRenamed { from, to } => {
                                    area.rename_files(from, to, cx)
                                }
                                PanelEvent::PathDeleted(path) => {
                                    area.close_deleted_files(path, window, cx)
                                }
                            })
                            .ok();
                    },
                ),
            );
            SidebarPanel {
                panel,
                view,
                _subscription: subscription,
            }
        })
        .collect()
}
