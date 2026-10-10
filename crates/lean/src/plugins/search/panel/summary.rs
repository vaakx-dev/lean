use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SearchPanel;
use super::running::SearchStatus;

impl SearchPanel {
    pub(super) fn render_summary(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let theme = cx.theme();
        let searching = matches!(self.status, SearchStatus::Searching);
        let message = match &self.status {
            SearchStatus::Idle => return None,
            SearchStatus::Failed(message) => {
                return Some(
                    div()
                        .flex_shrink_0()
                        .px_3()
                        .pb_2()
                        .text_xs()
                        .text_color(theme.danger)
                        .child(message.clone())
                        .into_any_element(),
                );
            }
            SearchStatus::Searching if self.results.files.is_empty() => {
                SharedString::from("Searching…")
            }
            SearchStatus::Done if self.results.files.is_empty() => {
                SharedString::from("No results found.")
            }
            SearchStatus::Searching | SearchStatus::Done => self.result_count().into(),
        };
        let has_results = !self.results.files.is_empty();
        let collapse_icon = if self.all_collapsed() {
            "icons/chevrons-up-down.svg"
        } else {
            "icons/list-collapse.svg"
        };
        let collapse_tooltip = if self.all_collapsed() {
            "Expand All"
        } else {
            "Collapse All"
        };
        let limit = self.config.max_results;
        let summary = v_flex()
            .flex_shrink_0()
            .pl_3()
            .pr_2()
            .pb_1()
            .child(
                h_flex()
                    .h(px(22.))
                    .gap_1p5()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .when(searching, |row| row.child(Spinner::new().xsmall()))
                    .child(div().flex_1().min_w_0().truncate().child(message))
                    .when(has_results, |row| {
                        row.child(
                            action_button("toggle-all", collapse_icon, collapse_tooltip)
                                .on_click(cx.listener(|panel, _, _, cx| panel.toggle_all(cx))),
                        )
                    })
                    .child(
                        action_button("refresh", "icons/refresh-cw.svg", "Refresh")
                            .on_click(cx.listener(|panel, _, _, cx| panel.search_now(cx))),
                    )
                    .child(
                        action_button("clear", "icons/x.svg", "Clear Search Results").on_click(
                            cx.listener(|panel, _, window, cx| panel.clear_query(window, cx)),
                        ),
                    ),
            )
            .when(self.results.truncated, |summary| {
                summary.child(div().text_xs().text_color(theme.warning).child(format!(
                    "Showing the first {limit} results. Narrow your search to see more."
                )))
            });
        Some(summary.into_any_element())
    }

    fn result_count(&self) -> String {
        let matches = self.results.match_count;
        let files = self.results.files.len();
        format!(
            "{matches} {} in {files} {}",
            plural(matches, "result", "results"),
            plural(files, "file", "files")
        )
    }

    fn clear_query(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.query_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.clear_results(cx);
    }
}

fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

fn action_button(id: &'static str, icon: &'static str, tooltip: &'static str) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .tooltip(tooltip)
}
