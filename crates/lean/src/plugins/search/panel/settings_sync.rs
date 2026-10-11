use gpui_kit::{Context, Subscription};

use super::SearchPanel;
use crate::plugins::search::config::SearchConfig;
use crate::plugins::search::settings::{MATCH_CASE, REGEX, SHOW_FILTERS, WHOLE_WORD};
use crate::settings::Settings;

impl SearchPanel {
    pub(super) fn observe_settings(cx: &mut Context<Self>) -> [Subscription; 5] {
        [
            MATCH_CASE.observe(cx, |panel, value, cx| {
                panel.adopt(|panel| &mut panel.options.match_case, value, cx)
            }),
            WHOLE_WORD.observe(cx, |panel, value, cx| {
                panel.adopt(|panel| &mut panel.options.whole_word, value, cx)
            }),
            REGEX.observe(cx, |panel, value, cx| {
                panel.adopt(|panel| &mut panel.options.regex, value, cx)
            }),
            SHOW_FILTERS.observe(cx, |panel, value, cx| {
                panel.adopt(|panel| &mut panel.show_filters, value, cx)
            }),
            cx.observe_global::<Settings>(|panel, cx| {
                let config = SearchConfig::from_settings(cx);
                if config != panel.config {
                    panel.config = config;
                    panel.rerun(cx);
                }
            }),
        ]
    }

    fn adopt(
        &mut self,
        toggle: impl FnOnce(&mut Self) -> &mut bool,
        value: bool,
        cx: &mut Context<Self>,
    ) {
        let current = toggle(self);
        if *current == value {
            return;
        }
        *current = value;
        self.rerun(cx);
    }

    fn rerun(&mut self, cx: &mut Context<Self>) {
        if self.query_input.read(cx).value().is_empty() {
            cx.notify();
        } else {
            self.search_now(cx);
        }
    }
}
