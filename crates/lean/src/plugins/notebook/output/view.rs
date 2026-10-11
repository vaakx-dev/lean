use gpui_kit::component::text::TextView;
use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{Output, OutputContent};

impl Output {
    pub(crate) fn render(&self, id: ElementId, cx: &App) -> AnyElement {
        self.content.render(id, cx)
    }
}

impl OutputContent {
    fn render(&self, id: ElementId, cx: &App) -> AnyElement {
        let theme = cx.theme();
        match self {
            OutputContent::Text(text) => text_block(text.clone(), cx).into_any_element(),
            OutputContent::Stream {
                text,
                stderr: false,
            } => text_block(text.display(), cx).into_any_element(),
            OutputContent::Stream { text, stderr: true } => text_block(text.display(), cx)
                .px_2()
                .py_1()
                .rounded(theme.radius)
                .bg(theme.danger.opacity(0.08))
                .into_any_element(),
            OutputContent::Error { title, traceback } => v_flex()
                .gap_1()
                .px_2()
                .py_1p5()
                .rounded(theme.radius)
                .bg(theme.danger.opacity(0.08))
                .child(
                    div()
                        .font_family(theme.mono_font_family.clone())
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.danger)
                        .child(title.clone()),
                )
                .when(!traceback.is_empty(), |error| {
                    error
                        .child(text_block(traceback.clone(), cx).text_color(theme.muted_foreground))
                })
                .into_any_element(),
            OutputContent::Image(image) => div()
                .child(img(image.clone()).max_w_full().rounded(theme.radius))
                .into_any_element(),
            OutputContent::Markdown(markdown) => TextView::markdown(id, markdown.clone())
                .selectable(true)
                .into_any_element(),
            OutputContent::Html(html) => TextView::html(id, html.clone())
                .selectable(true)
                .into_any_element(),
            OutputContent::Unsupported(message) => div()
                .text_sm()
                .italic()
                .text_color(theme.muted_foreground)
                .child(message.clone())
                .into_any_element(),
        }
    }
}

fn text_block(text: SharedString, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .w_full()
        .font_family(theme.mono_font_family.clone())
        .text_sm()
        .child(text)
}
