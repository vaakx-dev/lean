use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SvgViewer;
use crate::plugins::media::settings::SvgMode;

impl SvgViewer {
    fn render_code(&self) -> Div {
        div().flex_1().min_w_0().h_full().child(self.code.view())
    }

    fn render_preview(&self, cx: &Context<Self>) -> Stateful<Div> {
        let split = self.mode == SvgMode::Split;
        div()
            .id("svg-preview-pane")
            .flex_1()
            .min_w_0()
            .h_full()
            .when(split, |pane| {
                pane.border_l_1()
                    .border_color(cx.theme().border)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|viewer, _: &MouseDownEvent, window, cx| {
                            let focus_handle = viewer.focus_handle(cx);
                            window.focus(&focus_handle, cx);
                        }),
                    )
            })
            .child(self.preview.render(cx))
    }
}

impl Render for SvgViewer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx: &Context<Self> = cx;
        div()
            .id("svg-viewer")
            .relative()
            .flex()
            .size_full()
            .when(self.mode == SvgMode::Preview, |root| {
                root.track_focus(&self.focus_handle)
            })
            .when(self.mode.shows_code(), |root| {
                root.child(self.render_code())
            })
            .when(self.mode.shows_preview(), |root| {
                root.child(self.render_preview(cx))
            })
            .child(self.render_mode_switcher(cx))
    }
}
