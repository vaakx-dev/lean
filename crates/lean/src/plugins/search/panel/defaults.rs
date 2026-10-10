use gpui_kit::{App, Context};

use super::SearchPanel;
use crate::plugins::search::config::SearchConfig;
use crate::plugins::search::query::SearchOptions;
use crate::plugins::search::settings::SHOW_FILTERS;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct SearchDefaults {
    pub(super) options: SearchOptions,
    pub(super) show_filters: bool,
}

impl SearchDefaults {
    pub(super) fn from_settings(cx: &App) -> Self {
        Self {
            options: SearchOptions::from_settings(cx),
            show_filters: SHOW_FILTERS.get(cx),
        }
    }
}

impl SearchPanel {
    pub(super) fn apply_settings(&mut self, cx: &mut Context<Self>) {
        let defaults = SearchDefaults::from_settings(cx);
        let config = SearchConfig::from_settings(cx);
        let options_before = self.options;
        let filters_before = self.show_filters;
        if defaults != self.defaults {
            self.adopt_defaults(defaults);
        }
        let config_changed = config != self.config;
        self.config = config;
        let rerun =
            config_changed || self.options != options_before || self.show_filters != filters_before;
        if !rerun {
            return;
        }
        if self.query_input.read(cx).value().is_empty() {
            cx.notify();
        } else {
            self.search_now(cx);
        }
    }

    fn adopt_defaults(&mut self, defaults: SearchDefaults) {
        let before = std::mem::replace(&mut self.defaults, defaults);
        let options = &mut self.options;
        adopt(
            &mut options.match_case,
            before.options.match_case,
            defaults.options.match_case,
        );
        adopt(
            &mut options.whole_word,
            before.options.whole_word,
            defaults.options.whole_word,
        );
        adopt(
            &mut options.regex,
            before.options.regex,
            defaults.options.regex,
        );
        adopt(
            &mut self.show_filters,
            before.show_filters,
            defaults.show_filters,
        );
    }
}

fn adopt(current: &mut bool, before: bool, after: bool) {
    if before != after {
        *current = after;
    }
}
