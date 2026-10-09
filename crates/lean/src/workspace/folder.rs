use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui_kit::*;

use super::Workspace;
use super::panels::{SidebarPanel, build_panels};
use crate::editor::{EditorArea, EditorAreaEvent};
use crate::plugins::{FileDecorations, WorkspaceContext, status_items};
use crate::project::{FilesChanged, Project};
use crate::toolchain::{Toolchains, ToolchainsEvent};

const ACTIVATION_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

pub(super) struct ProjectViews {
    pub(super) context: WorkspaceContext,
    pub(super) panels: Vec<SidebarPanel>,
    pub(super) status_items: Vec<AnyView>,
    pub(super) subscriptions: Vec<Subscription>,
}

impl ProjectViews {
    pub(super) fn new(root: PathBuf, window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        let toolchains = cx.new(|cx| Toolchains::new(root.clone(), cx));
        let project = cx.new(|cx| Project::new(root, cx));
        let editor_area = cx.new(|cx| EditorArea::new(toolchains.clone(), cx));
        let decorations = cx.new(|_| FileDecorations::default());
        let context = WorkspaceContext::new(
            project.clone(),
            toolchains.clone(),
            editor_area.clone(),
            decorations,
        )
        .start_services(window, cx);
        let panels = build_panels(&context, window, cx);
        let status_items = status_items(cx)
            .to_vec()
            .into_iter()
            .map(|item| (item.build)(&context, window, cx))
            .collect();
        let subscriptions = vec![
            cx.subscribe_in(
                &project,
                window,
                |workspace, _, event: &FilesChanged, window, cx| {
                    workspace.sync_open_files(&event.0, window, cx)
                },
            ),
            cx.subscribe_in(
                &toolchains,
                window,
                |workspace, _, event: &ToolchainsEvent, window, cx| {
                    workspace.handle_toolchains_event(event, window, cx)
                },
            ),
            cx.subscribe(
                &editor_area,
                |workspace, _, event: &EditorAreaEvent, cx| match event {
                    EditorAreaEvent::ActiveEditorChanged => cx.notify(),
                    EditorAreaEvent::OpenFilesChanged => workspace.watch_open_files(cx),
                    _ => {}
                },
            ),
        ];
        Self {
            context,
            panels,
            status_items,
            subscriptions,
        }
    }
}

impl Workspace {
    pub(super) fn set_project_root(
        &mut self,
        root: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let root = dunce::canonicalize(&root).unwrap_or_else(|error| {
            log::debug!("failed to canonicalize {}: {error}", root.display());
            root
        });
        let views = ProjectViews::new(root, window, cx);
        self.context = views.context;
        self.panels = views.panels;
        self.status_items = views.status_items;
        self._project_subscriptions = views.subscriptions;
        self.reset_terminal(window, cx);
        self.active_panel = 0;
        self.sidebar_visible = true;
        self.focus_sidebar(window, cx);
        self.update_title(window, cx);
        self.remember_project(cx);
        cx.notify();
    }

    pub(super) fn refresh_after_activation(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !window.is_window_active()
            || self.last_activation_refresh.elapsed() < ACTIVATION_REFRESH_INTERVAL
        {
            return;
        }
        self.last_activation_refresh = Instant::now();
        self.context
            .project
            .update(cx, |project, cx| project.refresh_after_activation(cx));
    }
}
