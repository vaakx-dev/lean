use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, Theme, TitleBar, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use super::settings::TITLE_BAR_SEARCH_BOX;
use crate::app::actions::{OpenRecent, ToggleSidebar, ToggleTerminal};

const SEARCH_BOX_MAX_WIDTH: Pixels = px(560.);
const SEARCH_BOX_MIN_WINDOW_WIDTH: Pixels = px(760.);
const SEARCH_PANEL_ID: &str = "search";

pub(super) struct WorkspaceTitleBar {
    workspace: WeakEntity<Workspace>,
    _subscription: Subscription,
}

impl WorkspaceTitleBar {
    pub(super) fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            _subscription: cx.observe(&workspace, |_, _, cx| cx.notify()),
            workspace: workspace.downgrade(),
        }
    }
}

impl Render for WorkspaceTitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(workspace) = self.workspace.upgrade() else {
            return div().into_any_element();
        };
        workspace.update(cx, |workspace, cx| {
            workspace.render_title_bar(window, cx).into_any_element()
        })
    }
}

impl Workspace {
    fn render_title_bar(&self, window: &Window, cx: &Context<Self>) -> TitleBar {
        let theme = cx.theme();
        let project_name = self.context.project.read(cx).name().clone();
        let shows_search_box = TITLE_BAR_SEARCH_BOX.get(cx)
            && window.viewport_size().width >= SEARCH_BOX_MIN_WINDOW_WIDTH;
        let shows_sidebar = self.shows_sidebar();
        TitleBar::new()
            .on_close_window(Self::close_window_handler(cx))
            .bg(theme.background)
            .border_b_0()
            .pl_2()
            .child(
                h_flex()
                    .flex_1()
                    .gap_1()
                    .child(img("images/lean.png").size(px(20.)).mx_1().flex_shrink_0())
                    .child(self.app_menu_bar.clone()),
            )
            .child(
                h_flex()
                    .flex_basis(SEARCH_BOX_MAX_WIDTH)
                    .flex_shrink_1()
                    .min_w_0()
                    .max_w(SEARCH_BOX_MAX_WIDTH)
                    .px_2()
                    .when(shows_search_box, |center| {
                        center.child(
                            search_box(project_name, self.search_box_tooltip(), theme).on_click(
                                cx.listener(|workspace, _, window, cx| {
                                    workspace.activate_search(window, cx)
                                }),
                            ),
                        )
                    }),
            )
            .child(
                h_flex()
                    .flex_1()
                    .justify_end()
                    .gap_0p5()
                    .pr_1()
                    .child(layout_toggle(
                        "toggle-sidebar",
                        if shows_sidebar {
                            "icons/panel-left.svg"
                        } else {
                            "icons/panel-left-open.svg"
                        },
                        "Toggle Sidebar",
                        shows_sidebar,
                        ToggleSidebar,
                    ))
                    .child(layout_toggle(
                        "toggle-terminal",
                        if self.terminal_visible {
                            "icons/panel-bottom.svg"
                        } else {
                            "icons/panel-bottom-open.svg"
                        },
                        "Toggle Terminal",
                        self.terminal_visible,
                        ToggleTerminal,
                    )),
            )
    }

    fn has_search_panel(&self) -> bool {
        self.panel_index(SEARCH_PANEL_ID).is_some()
    }

    fn search_box_tooltip(&self) -> &'static str {
        if self.has_search_panel() {
            "Search in Files"
        } else {
            "Open Recent Folder"
        }
    }

    fn activate_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.has_search_panel() {
            self.show_panel(SEARCH_PANEL_ID, window, cx);
        } else {
            self.open_recent(&OpenRecent, window, cx);
        }
    }
}

fn search_box(project_name: SharedString, tooltip: &'static str, theme: &Theme) -> Button {
    Button::new("search-box")
        .ghost()
        .tooltip(tooltip)
        .w_full()
        .min_w_0()
        .h(px(26.))
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.muted)
        .child(
            h_flex()
                .w_full()
                .min_w_0()
                .justify_center()
                .gap_2()
                .text_color(theme.muted_foreground)
                .child(
                    Icon::empty()
                        .path("icons/search.svg")
                        .size_4()
                        .flex_shrink_0(),
                )
                .child(div().min_w_0().truncate().child(project_name)),
        )
}

fn layout_toggle(
    id: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    is_on: bool,
    action: impl Action,
) -> Button {
    Button::new(id)
        .ghost()
        .small()
        .icon(Icon::empty().path(icon))
        .tooltip_with_action(tooltip, &action, None)
        .toggled(is_on)
        .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
}
