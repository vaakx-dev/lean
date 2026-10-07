use std::path::PathBuf;

use gpui_kit::*;

use super::Workspace;
use super::sidebar::SidebarView;
use crate::explorer::{Explorer, ExplorerEvent};
use crate::project::{Project, ProjectEvent};
use crate::source_control::{SourceControl, SourceControlEvent};

pub(super) struct ProjectViews {
    pub(super) project: Entity<Project>,
    pub(super) explorer: Entity<Explorer>,
    pub(super) source_control: Entity<SourceControl>,
    pub(super) subscriptions: Vec<Subscription>,
}

impl ProjectViews {
    pub(super) fn new(root: PathBuf, window: &mut Window, cx: &mut Context<Workspace>) -> Self {
        let project = cx.new(|cx| Project::new(root, cx));
        let explorer = cx.new(|cx| Explorer::new(project.clone(), cx));
        let source_control = cx.new(|cx| SourceControl::new(project.clone(), window, cx));
        let subscriptions = vec![
            cx.subscribe(&project, |_, _, event: &ProjectEvent, cx| match event {
                ProjectEvent::GitChanged => cx.notify(),
            }),
            cx.subscribe_in(
                &explorer,
                window,
                |workspace, _, event: &ExplorerEvent, window, cx| match event {
                    ExplorerEvent::OpenFile(path) => workspace.open_file(path.clone(), window, cx),
                },
            ),
            cx.subscribe_in(
                &source_control,
                window,
                |workspace, _, event: &SourceControlEvent, window, cx| match event {
                    SourceControlEvent::OpenFile(path) => {
                        workspace.open_file(path.clone(), window, cx)
                    }
                },
            ),
        ];
        Self {
            project,
            explorer,
            source_control,
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
        self.project = views.project;
        self.explorer = views.explorer;
        self.source_control = views.source_control;
        self._project_subscriptions = views.subscriptions;
        let (editor_area, editor_subscription) = Self::create_editor_area(cx);
        self.editor_area = editor_area;
        self._editor_subscription = editor_subscription;
        self.reset_terminal(window, cx);
        self.sidebar_view = SidebarView::Explorer;
        self.sidebar_visible = true;
        self.focus_sidebar(window, cx);
        self.update_title(window, cx);
        cx.notify();
    }
}
