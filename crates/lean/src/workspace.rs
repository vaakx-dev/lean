mod activity_bar;
mod close_confirmation;
mod commands;
mod folder;
mod folder_picker;
mod layout;
mod panels;
mod persistence;
mod project_switch;
mod quick_settings;
mod recent;
pub(crate) mod settings;
mod shortcuts;
mod sidebar;
mod sidebar_resize;
mod status_bar;
mod terminal_panel;
mod title_bar;
mod toolchains;
mod window_close;

use std::path::{Path, PathBuf};
use std::time::Instant;

use gpui_kit::component::menu::AppMenuBar;
use gpui_kit::*;

use crate::plugins::WorkspaceContext;
use crate::recent_projects::settings::{MENU_LIMIT, RECENT_LIMIT};
use crate::watcher::FsChanges;
use folder::ProjectViews;
use panels::SidebarPanel;
use settings::{SIDEBAR_WIDTH, STATUS_BAR_VISIBLE, TITLE_BAR_SEARCH_BOX};
use sidebar_resize::clamp_sidebar_width;
use terminal_panel::TerminalDock;

pub(crate) use close_confirmation::CloseIntent;
pub(crate) use persistence::restored_window_bounds;
pub(crate) use status_bar::status_item;

const GAP: Pixels = px(6.);

pub(crate) struct Workspace {
    context: WorkspaceContext,
    panels: Vec<SidebarPanel>,
    status_items: Vec<AnyView>,
    app_menu_bar: Entity<AppMenuBar>,
    focus_handle: FocusHandle,
    active_panel: usize,
    sidebar_visible: bool,
    sidebar_width: Pixels,
    resizing_sidebar: bool,
    terminal_visible: bool,
    terminal: Option<TerminalDock>,
    last_activation_refresh: Instant,
    confirming_close: bool,
    close_confirmed: bool,
    _project_subscriptions: Vec<Subscription>,
    _activation_subscription: Subscription,
    _settings_subscriptions: Vec<Subscription>,
}

impl Workspace {
    pub(crate) fn new(
        project_root: PathBuf,
        file: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let views = ProjectViews::new(project_root, window, cx);
        let activation_subscription = cx
            .observe_window_activation(window, |workspace, window, cx| {
                workspace.refresh_after_activation(window, cx)
            });
        let mut workspace = Self {
            context: views.context,
            panels: views.panels,
            status_items: views.status_items,
            app_menu_bar: AppMenuBar::new(cx),
            focus_handle: cx.focus_handle(),
            active_panel: 0,
            sidebar_visible: true,
            sidebar_width: clamp_sidebar_width(SIDEBAR_WIDTH.get(cx)),
            resizing_sidebar: false,
            terminal_visible: false,
            terminal: None,
            last_activation_refresh: Instant::now(),
            confirming_close: false,
            close_confirmed: false,
            _project_subscriptions: views.subscriptions,
            _activation_subscription: activation_subscription,
            _settings_subscriptions: Self::observe_settings(cx),
        };
        workspace.active_panel = workspace.default_panel_index(cx);
        workspace.restore_layout(window, cx);
        workspace.update_title(window, cx);
        workspace.remember_project(cx);
        workspace.reopen_files(file, window, cx);
        Self::guard_window_close(window, cx);
        workspace
    }

    fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            SIDEBAR_WIDTH.observe(cx, |workspace, width, cx| {
                workspace.sidebar_width = clamp_sidebar_width(width);
                cx.notify();
            }),
            STATUS_BAR_VISIBLE.observe(cx, |_, _, cx| cx.notify()),
            TITLE_BAR_SEARCH_BOX.observe(cx, |_, _, cx| cx.notify()),
            RECENT_LIMIT.observe(cx, |workspace, _, cx| workspace.refresh_app_menus(cx)),
            MENU_LIMIT.observe(cx, |workspace, _, cx| workspace.refresh_app_menus(cx)),
        ]
    }

    fn project_root(&self, cx: &App) -> PathBuf {
        self.context.project.read(cx).root().to_path_buf()
    }

    fn update_title(&self, window: &mut Window, cx: &App) {
        let name = self.context.project.read(cx).name().clone();
        window.set_window_title(&format!("{name} — Lean"));
    }

    fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.open(path, window, cx));
    }

    fn open_file_at(
        &mut self,
        path: PathBuf,
        line: usize,
        column: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.open_at(path, line, column, window, cx));
    }

    fn rename_open_files(&mut self, from: &Path, to: &Path, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.rename_files(from, to, cx));
    }

    fn close_deleted_files(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.close_deleted_files(path, window, cx));
    }

    fn sync_open_files(
        &mut self,
        changes: &FsChanges,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context
            .editor_area
            .update(cx, |area, cx| area.apply_fs_changes(changes, window, cx));
    }

    fn watch_open_files(&mut self, cx: &mut Context<Self>) {
        let paths = self.context.editor_area.read(cx).open_paths();
        self.context
            .project
            .update(cx, |project, _| project.watch_open_files(paths));
    }
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
