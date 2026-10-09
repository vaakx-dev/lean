use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::Input;
use gpui_kit::component::{Icon, Selectable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SearchPanel;
use crate::plugins::search::query::SearchOptions;

impl SearchPanel {
    pub(super) fn render_query_bar(&self, cx: &Context<Self>) -> Div {
        v_flex()
            .flex_shrink_0()
            .gap_1p5()
            .px_2()
            .pb_2()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        div().flex_1().min_w_0().child(
                            Input::new(&self.query_input)
                                .small()
                                .cleanable(true)
                                .suffix(self.render_option_toggles(cx)),
                        ),
                    )
                    .child(
                        toggle_button(
                            "toggle-filters",
                            "icons/ellipsis.svg",
                            "Toggle Search Details",
                            self.show_filters,
                        )
                        .on_click(cx.listener(|panel, _, _, cx| panel.toggle_filters(cx))),
                    ),
            )
            .when(self.show_filters, |bar| {
                bar.child(Input::new(&self.include_input).small())
                    .child(Input::new(&self.exclude_input).small())
            })
    }

    fn render_option_toggles(&self, cx: &Context<Self>) -> Div {
        let options = self.options;
        h_flex()
            .gap(px(2.))
            .child(
                toggle_button(
                    "match-case",
                    "icons/case-sensitive.svg",
                    "Match Case",
                    options.match_case,
                )
                .on_click(
                    cx.listener(|panel, _, _, cx| panel.update_options(toggle_match_case, cx)),
                ),
            )
            .child(
                toggle_button(
                    "whole-word",
                    "icons/whole-word.svg",
                    "Match Whole Word",
                    options.whole_word,
                )
                .on_click(
                    cx.listener(|panel, _, _, cx| panel.update_options(toggle_whole_word, cx)),
                ),
            )
            .child(
                toggle_button(
                    "regex",
                    "icons/regex.svg",
                    "Use Regular Expression",
                    options.regex,
                )
                .on_click(cx.listener(|panel, _, _, cx| panel.update_options(toggle_regex, cx))),
            )
    }
}

fn toggle_button(
    id: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    selected: bool,
) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .tab_stop(false)
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
        .selected(selected)
}

fn toggle_match_case(options: &mut SearchOptions) {
    options.match_case = !options.match_case;
}

fn toggle_whole_word(options: &mut SearchOptions) {
    options.whole_word = !options.whole_word;
}

fn toggle_regex(options: &mut SearchOptions) {
    options.regex = !options.regex;
}
