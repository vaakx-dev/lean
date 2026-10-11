use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Selectable as _, Sizable as _, h_flex};
use gpui_kit::*;

use super::ImageViewer;
use super::zoom::Zoom;
use crate::ui::icon_button;

const SCALE_LABEL_WIDTH: Pixels = px(48.);

impl ImageViewer {
    pub(super) fn render_toolbar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .absolute()
            .top_2()
            .right_4()
            .gap_0p5()
            .p_0p5()
            .rounded(theme.radius)
            .border_1()
            .border_color(theme.border)
            .bg(theme.popover)
            .shadow_sm()
            .occlude()
            .child(
                icon_button("zoom-out", "icons/zoom-out.svg", "Zoom Out")
                    .on_click(cx.listener(|viewer, _, _, cx| viewer.zoom_out(cx))),
            )
            .child(
                Button::new("actual-size")
                    .ghost()
                    .xsmall()
                    .label(self.scale_label())
                    .tooltip("Actual Size")
                    .w(SCALE_LABEL_WIDTH)
                    .on_click(cx.listener(|viewer, _, _, cx| viewer.zoom_to_actual_size(cx))),
            )
            .child(
                icon_button("zoom-in", "icons/zoom-in.svg", "Zoom In")
                    .on_click(cx.listener(|viewer, _, _, cx| viewer.zoom_in(cx))),
            )
            .child(
                icon_button("zoom-to-fit", "icons/maximize.svg", "Fit to View")
                    .selected(self.zoom == Zoom::Fit)
                    .on_click(cx.listener(|viewer, _, _, cx| viewer.zoom_to_fit(cx))),
            )
    }

    fn scale_label(&self) -> SharedString {
        match self.current_scale() {
            Some(scale) => format!("{:.0}%", scale * 100.).into(),
            None => "Fit".into(),
        }
    }
}
