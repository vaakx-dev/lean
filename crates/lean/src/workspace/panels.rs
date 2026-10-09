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
            let workspace = cx.weak_entity();
            let subscription = view.subscribe(
                window,
                cx,
                Box::new(
                    move |event: &PanelEvent, window: &mut Window, cx: &mut App| {
                        let update = workspace.update(cx, |workspace, cx| {
                            workspace.handle_panel_event(event, window, cx)
                        });
                        if let Err(error) = update {
                            log::debug!(
                                "workspace dropped while handling a panel event: {error:#}"
                            );
                        }
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

impl Workspace {
    fn handle_panel_event(
        &mut self,
        event: &PanelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            PanelEvent::OpenFile(path) => self.open_file(path.clone(), window, cx),
            PanelEvent::OpenFileAt { path, line, column } => {
                self.open_file_at(path.clone(), *line, *column, window, cx)
            }
            PanelEvent::PathRenamed { from, to } => self.rename_open_files(from, to, cx),
            PanelEvent::PathDeleted(path) => self.close_deleted_files(path, window, cx),
        }
    }
}
