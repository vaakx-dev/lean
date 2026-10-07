use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{TerminalPanel, TerminalPanelEvent, TerminalTab};

const HEADER_HEIGHT: Pixels = px(32.);
const TAB_GROUP: &str = "terminal-tab";

impl TerminalPanel {
    pub(super) fn render_header(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .h(HEADER_HEIGHT)
            .flex_shrink_0()
            .pl_3()
            .pr_1()
            .gap_2()
            .border_b_1()
            .border_color(theme.border)
            .text_sm()
            .child(
                Icon::empty()
                    .path("icons/square-terminal.svg")
                    .size_4()
                    .flex_shrink_0(),
            )
            .child(self.render_titles(cx))
            .child(
                header_button("new-terminal", "icons/plus.svg", "New Terminal")
                    .on_click(cx.listener(|panel, _, window, cx| panel.new_terminal(window, cx))),
            )
            .child(
                header_button("hide-terminal", "icons/x.svg", "Hide Terminal (Ctrl+`)")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(TerminalPanelEvent::Hide))),
            )
    }

    fn render_titles(&self, cx: &Context<Self>) -> Stateful<Div> {
        let titles = h_flex()
            .id("terminal-tabs")
            .flex_1()
            .min_w_0()
            .h_full()
            .gap_1()
            .overflow_x_scroll();
        match self.tabs.as_slice() {
            [tab] => titles.child(
                div()
                    .min_w_0()
                    .truncate()
                    .child(tab.view.read(cx).shell_name()),
            ),
            tabs => titles.children(
                tabs.iter()
                    .enumerate()
                    .map(|(index, tab)| self.render_tab(index, tab, cx)),
            ),
        }
    }

    fn render_tab(&self, index: usize, tab: &TerminalTab, cx: &Context<Self>) -> Stateful<Div> {
        let theme = cx.theme();
        let is_active = index == self.active;
        let title = format!("{} {}", tab.view.read(cx).shell_name(), index + 1);
        h_flex()
            .id(index)
            .group(TAB_GROUP)
            .h(px(24.))
            .flex_shrink_0()
            .pl_2()
            .pr_0p5()
            .gap_1()
            .rounded(theme.radius)
            .cursor_pointer()
            .text_color(theme.muted_foreground)
            .when(is_active, |element| {
                element.bg(theme.accent).text_color(theme.foreground)
            })
            .when(!is_active, |element| {
                element.hover(|style| style.bg(theme.list_hover))
            })
            .on_click(cx.listener(move |panel, _, window, cx| panel.activate(index, window, cx)))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |panel, _, window, cx| panel.close(index, window, cx)),
            )
            .child(title)
            .child(self.render_close_button(index, is_active, cx))
    }

    fn render_close_button(
        &self,
        index: usize,
        is_active: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        div()
            .id("close")
            .size(px(18.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius)
            .hover(|style| style.bg(theme.list_hover))
            .on_click(cx.listener(move |panel, _, window, cx| {
                cx.stop_propagation();
                panel.close(index, window, cx)
            }))
            .child(
                div()
                    .flex()
                    .when(!is_active, |icon| icon.invisible())
                    .group_hover(TAB_GROUP, |style| style.visible())
                    .child(Icon::empty().path("icons/x.svg").size(px(12.))),
            )
    }
}

fn header_button(id: &'static str, icon: &'static str, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
}
