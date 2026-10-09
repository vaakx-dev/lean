use std::sync::Arc;

use anyhow::Result;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme as _, Icon, h_flex, v_flex};
use gpui_kit::*;

use crate::plugins::media::picture::{Fit, picture};

const PADDING: Pixels = px(16.);

#[derive(Default)]
pub(super) struct SvgPreview {
    image: Option<Arc<RenderImage>>,
    error: Option<SharedString>,
}

impl SvgPreview {
    pub(super) fn show(&mut self, rendered: Result<Arc<RenderImage>>, cx: &mut App) {
        match rendered {
            Ok(image) => {
                self.error = None;
                if let Some(previous) = self.image.replace(image) {
                    cx.drop_image(previous, None);
                }
            }
            Err(error) => self.error = Some(format!("{error:#}").into()),
        }
    }

    pub(super) fn release(&mut self, cx: &mut App) {
        if let Some(image) = self.image.take() {
            cx.drop_image(image, None);
        }
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        let content = match &self.image {
            Some(image) => picture("svg-preview", image, Fit::Contain, cx)
                .size_full()
                .into_any_element(),
            None => empty_preview(self.error.is_some(), cx).into_any_element(),
        };
        div()
            .relative()
            .size_full()
            .p(PADDING)
            .bg(cx.theme().background)
            .child(content)
            .children(
                self.image
                    .as_ref()
                    .and(self.error.clone())
                    .map(|error| invalid_badge(error, cx)),
            )
    }
}

fn empty_preview(invalid: bool, cx: &App) -> Div {
    let theme = cx.theme();
    let message = if invalid {
        "This SVG can't be rendered."
    } else {
        "Rendering preview…"
    };
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            Icon::empty()
                .path("icons/file-image.svg")
                .size(px(48.))
                .text_color(theme.muted_foreground),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(message),
        )
}

fn invalid_badge(error: SharedString, cx: &App) -> Stateful<Div> {
    let theme = cx.theme();
    h_flex()
        .id("invalid-svg")
        .absolute()
        .left_2()
        .bottom_2()
        .gap_1()
        .px_2()
        .py_0p5()
        .rounded(theme.radius)
        .bg(theme.danger.opacity(0.12))
        .text_xs()
        .text_color(theme.danger)
        .child(Icon::empty().path("icons/triangle-alert.svg").size_3())
        .child("Invalid SVG")
        .tooltip(move |window, cx| Tooltip::new(error.clone()).build(window, cx))
}
