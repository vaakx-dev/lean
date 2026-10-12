use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::commands::bind_commands;
use super::settings::STATUS_BAR_VISIBLE;
use super::sidebar_resize::sidebar_resize_handle;
use super::{GAP, Workspace};

impl Workspace {
    fn render_body(&self, show_status_bar: bool, window: &Window, cx: &Context<Self>) -> Div {
        h_flex()
            .flex_1()
            .min_h_0()
            .pt(GAP)
            .pr(GAP)
            .when(!show_status_bar, |body| body.pb(GAP))
            .child(self.render_activity_bar(cx))
            .when(self.shows_sidebar(), |body| {
                body.child(
                    self.render_sidebar(self.visible_sidebar_width(window), cx)
                        .on_drag_move(cx.listener(Self::resize_sidebar)),
                )
                .child(sidebar_resize_handle())
            })
            .child(self.render_editor_column(cx))
    }

    fn render_editor_column(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .gap(GAP)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.sidebar)
                    .child(
                        self.context
                            .editor_area
                            .clone()
                            .cached(StyleRefinement::default().size_full()),
                    ),
            )
            .children(self.render_terminal_panel(cx))
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let window: &Window = window;
        let cx: &Context<Self> = cx;
        let theme = cx.theme();
        let show_status_bar = STATUS_BAR_VISIBLE.get(cx);
        let root = v_flex()
            .id("workspace")
            .key_context("Workspace")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground);
        bind_commands(root, cx)
            .child(self.render_title_bar(window, cx))
            .child(self.render_body(show_status_bar, window, cx))
            .when(show_status_bar, |root| root.child(self.render_status_bar()))
    }
}
