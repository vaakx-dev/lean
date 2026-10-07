use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, Theme, TitleBar, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Workspace;
use crate::app::actions::{ToggleSidebar, ToggleTerminal};

const SEARCH_BOX_MAX_WIDTH: Pixels = px(560.);
const SEARCH_BOX_MIN_WINDOW_WIDTH: Pixels = px(760.);

impl Workspace {
    pub(super) fn render_title_bar(&self, window: &Window, cx: &Context<Self>) -> TitleBar {
        let theme = cx.theme();
        let project_name = self.project.read(cx).name().clone();
        let has_room_for_search = window.viewport_size().width >= SEARCH_BOX_MIN_WINDOW_WIDTH;
        TitleBar::new()
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
                    .when(has_room_for_search, |center| {
                        center.child(search_box(project_name, theme))
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
                        if self.sidebar_visible {
                            "icons/panel-left.svg"
                        } else {
                            "icons/panel-left-open.svg"
                        },
                        "Toggle Sidebar (Ctrl+B)",
                        self.sidebar_visible,
                        ToggleSidebar,
                    ))
                    .child(layout_toggle(
                        "toggle-terminal",
                        if self.terminal_visible {
                            "icons/panel-bottom.svg"
                        } else {
                            "icons/panel-bottom-open.svg"
                        },
                        "Toggle Terminal (Ctrl+`)",
                        self.terminal_visible,
                        ToggleTerminal,
                    )),
            )
    }
}

fn search_box(project_name: SharedString, theme: &Theme) -> Button {
    Button::new("search-box")
        .ghost()
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
        .tooltip(tooltip)
        .toggled(is_on)
        .on_click(move |_, window, cx| window.dispatch_action(action.boxed_clone(), cx))
}
