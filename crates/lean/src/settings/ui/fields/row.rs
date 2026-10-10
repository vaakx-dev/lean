use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::setting::RenderOptions;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, Theme, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::field::Field;

const MARKER_SIZE: Pixels = px(6.);
const RESET_SLOT: Pixels = px(22.);

impl Field {
    pub(super) fn render(
        &self,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let enabled = self.enabled.as_ref().is_none_or(|enabled| enabled(cx));
        let stacked = self.stacked || options.layout() == Axis::Vertical;
        let options = options
            .with_disabled(options.is_disabled() || !enabled)
            .with_layout(if stacked {
                Axis::Vertical
            } else {
                Axis::Horizontal
            });
        let modified = (self.modified)(cx);
        let control = (self.control)(&options, window, cx);
        let theme = cx.theme();
        let label = self.render_label(theme);
        let reset = self.render_reset_slot(modified);
        let row = if stacked {
            v_flex()
                .gap_2()
                .child(h_flex().gap_3().items_start().child(label).child(reset))
                .child(div().w_full().child(control))
        } else {
            h_flex()
                .gap_3()
                .child(label)
                .child(div().flex_shrink_0().child(control))
                .child(reset)
        };
        row.relative()
            .w_full()
            .pl_4()
            .when(!enabled, |row| row.opacity(0.5))
            .when(modified, |row| row.child(modified_marker(stacked, theme)))
            .into_any_element()
    }

    fn render_label(&self, theme: &Theme) -> Div {
        v_flex()
            .flex_1()
            .min_w_0()
            .gap_0p5()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(div().text_sm().child(self.title.clone()))
                    .when(self.restart, |title| title.child(restart_badge(theme))),
            )
            .when(!self.description.is_empty(), |label| {
                label.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(self.description.clone()),
                )
            })
    }

    fn render_reset_slot(&self, modified: bool) -> Div {
        let reset = self.reset.clone();
        div().w(RESET_SLOT).flex_shrink_0().when(modified, |slot| {
            slot.child(
                Button::new(SharedString::from(format!("reset-{}", self.id)))
                    .ghost()
                    .xsmall()
                    .icon(Icon::empty().path("icons/rotate-ccw.svg"))
                    .tooltip("Reset to default")
                    .on_click(move |_, _, cx| reset(cx)),
            )
        })
    }
}

fn modified_marker(stacked: bool, theme: &Theme) -> Div {
    let marker = div().size(MARKER_SIZE).rounded_full().bg(theme.primary);
    let column = div().absolute().left(px(4.)).top_0();
    if stacked {
        column.pt(px(7.)).child(marker)
    } else {
        column.bottom_0().flex().items_center().child(marker)
    }
}

fn restart_badge(theme: &Theme) -> Div {
    h_flex()
        .gap_1()
        .text_xs()
        .text_color(theme.warning)
        .child(Icon::empty().path("icons/rotate-cw.svg").size_3())
        .child("Requires restart")
}
