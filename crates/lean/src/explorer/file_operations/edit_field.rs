use gpui_kit::component::input::Input;
use gpui_kit::component::{Sizable as _, Theme};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::inline_edit::InlineEdit;

const ERROR_PRIORITY: usize = 1;

impl InlineEdit {
    pub(in crate::explorer) fn render_field(&self, theme: &Theme) -> Div {
        let has_error = self.error.is_some();
        div()
            .relative()
            .flex_1()
            .min_w_0()
            .child(
                Input::new(&self.input)
                    .small()
                    .text_size(theme.font_size)
                    .when(has_error, |input| {
                        input.focus_bordered(false).border_color(theme.danger)
                    }),
            )
            .when_some(self.error.clone(), |field, error| {
                field.child(error_message(error, theme))
            })
    }

    pub(in crate::explorer) fn draft_name(&self, cx: &App) -> SharedString {
        self.input.read(cx).value()
    }
}

fn error_message(error: SharedString, theme: &Theme) -> Div {
    div().absolute().top(relative(1.)).left_0().right_0().child(
        deferred(
            div()
                .px_1p5()
                .py_1()
                .text_xs()
                .rounded(theme.radius)
                .bg(theme.danger)
                .text_color(theme.danger_foreground)
                .child(error),
        )
        .with_priority(ERROR_PRIORITY),
    )
}
