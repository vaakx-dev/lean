mod changes;
mod decorations;
mod diff_views;
mod gutter;
mod refresh;

use std::path::PathBuf;

use gpui_kit::component::Theme;
use gpui_kit::*;

use super::diff::DiffView;
use super::repo::{GitReadError, GitState};
use crate::editor::{EditorArea, EditorAreaEvent};
use crate::plugins::git::settings::{
    EXPLORER_DECORATIONS, EXPLORER_FOLDER_DOTS, GUTTER_BASE, GUTTER_ENABLED, HISTORY_LIMIT,
    INCLUDE_UNTRACKED, SHOW_HISTORY, history_page_size,
};
use crate::plugins::{FileDecorations, WorkspaceContext};
use crate::project::{FilesChanged, Project, ProjectEvent};
use gutter::GutterSync;
use refresh::Refresh;

pub(crate) enum GitEvent {
    Changed,
}

pub(crate) struct GitProject {
    project: Entity<Project>,
    editor_area: Entity<EditorArea>,
    decorations: Entity<FileDecorations>,
    state: Option<GitState>,
    error: Option<GitReadError>,
    refresh: Refresh,
    gutter: GutterSync,
    diff_views: Vec<WeakEntity<DiffView>>,
    history_limit: usize,
    _subscriptions: [Subscription; 4],
    _settings: Vec<Subscription>,
}

impl EventEmitter<GitEvent> for GitProject {}

impl GitProject {
    pub(super) fn start(workspace: &WorkspaceContext, _: &mut Window, cx: &mut App) -> AnyEntity {
        cx.new(|cx| Self::new(workspace, cx)).into_any()
    }

    pub(super) fn for_workspace(workspace: &WorkspaceContext) -> Entity<Self> {
        workspace
            .service::<Self>()
            .expect("the git plugin starts a git project for every workspace")
    }

    fn new(workspace: &WorkspaceContext, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.subscribe(&workspace.project, |git, _, event: &FilesChanged, cx| {
                git.files_changed(&event.0, cx)
            }),
            cx.subscribe(&workspace.project, |git, _, event: &ProjectEvent, cx| {
                git.project_event(event, cx)
            }),
            cx.subscribe(
                &workspace.editor_area,
                |git, _, event: &EditorAreaEvent, cx| git.editor_event(event, cx),
            ),
            cx.observe_global::<Theme>(|git, cx| git.theme_changed(cx)),
        ];
        let mut git = Self {
            project: workspace.project.clone(),
            editor_area: workspace.editor_area.clone(),
            decorations: workspace.decorations.clone(),
            state: None,
            error: None,
            refresh: Refresh::default(),
            gutter: GutterSync::default(),
            diff_views: Vec::new(),
            history_limit: history_page_size(cx),
            _subscriptions: subscriptions,
            _settings: Self::observe_settings(cx),
        };
        git.start_read(cx);
        git
    }

    fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            INCLUDE_UNTRACKED.observe(cx, |git, _, cx| git.start_read(cx)),
            SHOW_HISTORY.observe(cx, |git, _, cx| git.start_read(cx)),
            HISTORY_LIMIT.observe(cx, |git, _, cx| {
                git.history_limit = history_page_size(cx);
                git.start_read(cx);
            }),
            GUTTER_ENABLED.observe(cx, |git, _, cx| git.repaint_gutter(cx)),
            GUTTER_BASE.observe(cx, |git, _, cx| git.gutter_base_changed(cx)),
            EXPLORER_DECORATIONS.observe(cx, |git, _, cx| git.publish_decorations(cx)),
            EXPLORER_FOLDER_DOTS.observe(cx, |git, _, cx| git.publish_decorations(cx)),
        ]
    }

    pub(crate) fn root(&self, cx: &App) -> PathBuf {
        self.project.read(cx).root().to_path_buf()
    }

    pub(crate) fn state(&self) -> Option<&GitState> {
        self.state.as_ref()
    }

    pub(crate) fn error(&self) -> Option<&GitReadError> {
        self.error.as_ref()
    }

    fn project_event(&mut self, event: &ProjectEvent, cx: &mut Context<Self>) {
        match event {
            ProjectEvent::Activated if self.state.is_some() => {}
            ProjectEvent::Activated => self.refresh(cx),
            ProjectEvent::FilesTouched => self.schedule_refresh(cx),
            ProjectEvent::RevealRequested(_) | ProjectEvent::WatchFailed(_) => {}
        }
    }

    fn editor_event(&mut self, event: &EditorAreaEvent, cx: &mut Context<Self>) {
        match event {
            EditorAreaEvent::OpenFilesChanged => self.schedule_gutter(cx),
            EditorAreaEvent::FileEdited(path) => self.refresh_gutter_for(path, cx),
            EditorAreaEvent::Saved => self.schedule_refresh(cx),
            _ => {}
        }
    }

    fn theme_changed(&mut self, cx: &mut Context<Self>) {
        self.publish_decorations(cx);
        self.repaint_gutter(cx);
    }
}
