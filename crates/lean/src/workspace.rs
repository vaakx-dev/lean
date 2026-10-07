mod activity_bar;
mod commands;
mod folder;
mod folder_picker;
mod layout;
mod sidebar;
mod sidebar_resize;
mod status_bar;
mod terminal_panel;
mod title_bar;

use std::path::{Path, PathBuf};

use gpui_kit::component::menu::AppMenuBar;
use gpui_kit::*;

use crate::editor::{EditorArea, EditorAreaEvent};
use crate::explorer::Explorer;
use crate::project::Project;
use crate::source_control::SourceControl;
use folder::ProjectViews;
use sidebar::SidebarView;
use terminal_panel::TerminalDock;

const GAP: Pixels = px(6.);

pub(crate) struct Workspace {
    project: Entity<Project>,
    explorer: Entity<Explorer>,
    source_control: Entity<SourceControl>,
    editor_area: Entity<EditorArea>,
    app_menu_bar: Entity<AppMenuBar>,
    focus_handle: FocusHandle,
    sidebar_view: SidebarView,
    sidebar_visible: bool,
    sidebar_width: Pixels,
    resizing_sidebar: bool,
    terminal_visible: bool,
    terminal: Option<TerminalDock>,
    _editor_subscription: Subscription,
    _project_subscriptions: Vec<Subscription>,
}

impl Workspace {
    pub(crate) fn new(project_root: PathBuf, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (editor_area, editor_subscription) = Self::create_editor_area(cx);
        let views = ProjectViews::new(project_root, window, cx);
        let workspace = Self {
            project: views.project,
            explorer: views.explorer,
            source_control: views.source_control,
            editor_area,
            app_menu_bar: AppMenuBar::new(cx),
            focus_handle: cx.focus_handle(),
            sidebar_view: SidebarView::Explorer,
            sidebar_visible: true,
            sidebar_width: px(260.),
            resizing_sidebar: false,
            terminal_visible: false,
            terminal: None,
            _editor_subscription: editor_subscription,
            _project_subscriptions: views.subscriptions,
        };
        workspace.update_title(window, cx);
        workspace
    }

    fn create_editor_area(cx: &mut Context<Self>) -> (Entity<EditorArea>, Subscription) {
        let editor_area = cx.new(EditorArea::new);
        let subscription = cx.subscribe(
            &editor_area,
            |workspace, _, event: &EditorAreaEvent, cx| match event {
                EditorAreaEvent::ActiveEditorChanged => cx.notify(),
                EditorAreaEvent::FileRecreated(path) => workspace.show_recreated_file(path, cx),
            },
        );
        (editor_area, subscription)
    }

    fn update_title(&self, window: &mut Window, cx: &App) {
        let name = self.project.read(cx).name().clone();
        window.set_window_title(&format!("{name} — Lean"));
    }

    fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_area
            .update(cx, |area, cx| area.open(path, window, cx));
    }

    fn rename_open_files(&mut self, from: &Path, to: &Path, cx: &mut Context<Self>) {
        self.editor_area
            .update(cx, |area, cx| area.rename_files(from, to, cx));
    }

    fn close_deleted_files(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.editor_area
            .update(cx, |area, cx| area.close_deleted_files(path, cx));
    }

    fn show_recreated_file(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.explorer
            .update(cx, |explorer, cx| explorer.show_recreated_file(path, cx));
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
