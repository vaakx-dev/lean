mod activity_bar;
mod close_confirmation;
mod commands;
mod folder;
mod layout;
mod open_project;
mod panels;
mod persistence;
mod quick_settings;
pub(crate) mod settings;
mod sidebar;
mod sidebar_resize;
mod status_bar;
mod terminal_panel;
mod title_bar;
mod toolchains;

use std::path::PathBuf;
use std::time::Instant;

use gpui_kit::component::menu::AppMenuBar;
use gpui_kit::*;

use crate::plugins::WorkspaceContext;
use crate::recent_projects::settings::{MENU_LIMIT, RECENT_LIMIT};
use folder::ProjectViews;
use panels::SidebarPanel;
use settings::{SIDEBAR_WIDTH, STATUS_BAR_VISIBLE, TITLE_BAR_SEARCH_BOX};
use sidebar_resize::clamp_sidebar_width;
use status_bar::WorkspaceStatusBar;
use terminal_panel::TerminalDock;

pub(crate) use close_confirmation::CloseIntent;
pub(crate) use persistence::restored_window_bounds;
pub(crate) use status_bar::status_item;

const GAP: Pixels = px(6.);

pub(crate) struct Workspace {
    context: WorkspaceContext,
    panels: Vec<SidebarPanel>,
    status_bar: Entity<WorkspaceStatusBar>,
    app_menu_bar: Entity<AppMenuBar>,
    focus_handle: FocusHandle,
    active_panel: usize,
    sidebar_visible: bool,
    sidebar_width: Pixels,
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
            status_bar: views.status_bar,
            app_menu_bar: AppMenuBar::new(cx),
            focus_handle: cx.focus_handle(),
            active_panel: 0,
            sidebar_visible: true,
            sidebar_width: clamp_sidebar_width(SIDEBAR_WIDTH.get(cx)),
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
}

impl Focusable for Workspace {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
