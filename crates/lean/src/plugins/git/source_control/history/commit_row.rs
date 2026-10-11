use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::HistoryEntry;
use super::graph_paint::graph_column;
use crate::plugins::git::source_control::SourceControl;
use crate::plugins::git::source_control::rows::{clickable_row, row_container};

const AUTHOR_MIN_LIST_WIDTH: Pixels = px(360.);
const AUTHOR_MAX_WIDTH: Pixels = px(110.);

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

impl SourceControl {
    pub(in crate::plugins::git::source_control) fn render_commit_row(
        &self,
        index: usize,
        entry_index: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let row = row_container(index, theme).pl_2().pr_2();
        let Some(entry) = self.history.entries.get(entry_index) else {
            return row.into_any_element();
        };
        let commit = &entry.commit;
        let tooltip = commit_tooltip(entry);
        let shows_author = self.list_width() >= AUTHOR_MIN_LIST_WIDTH;
        row.child(graph_column(Rc::clone(&entry.graph), self.history.lanes))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(commit.summary.clone()),
            )
            .when(shows_author, |row| {
                row.child(
                    div()
                        .flex_shrink(4.)
                        .min_w_0()
                        .max_w(AUTHOR_MAX_WIDTH)
                        .truncate()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(commit.author.clone()),
                )
            })
            .child(
                div()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(relative_time(commit.time)),
            )
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .into_any_element()
    }

    pub(in crate::plugins::git::source_control) fn render_load_more_row(
        &self,
        index: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        clickable_row(index, theme)
            .pl_2()
            .text_sm()
            .text_color(theme.muted_foreground)
            .hover(|style| style.bg(theme.list_hover).text_color(theme.foreground))
            .child("Load more commits…")
            .on_click(cx.listener(|source_control, _, _, cx| {
                source_control
                    .git
                    .update(cx, |git, cx| git.load_more_history(cx))
            }))
            .into_any_element()
    }

    fn list_width(&self) -> Pixels {
        self.scroll_handle
            .0
            .borrow()
            .last_item_size
            .map_or(px(0.), |size| size.item.width)
    }
}

fn commit_tooltip(entry: &HistoryEntry) -> SharedString {
    let commit = &entry.commit;
    format!(
        "{}\n\n{} · {}",
        commit.message, commit.author, commit.short_id
    )
    .into()
}

fn relative_time(timestamp: i64) -> String {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX),
        Err(error) => {
            log::warn!("the system clock is before the unix epoch: {error}");
            return String::new();
        }
    };
    let seconds = now.saturating_sub(timestamp).max(0);
    let (amount, unit) = match seconds {
        seconds if seconds < MINUTE => return "now".to_string(),
        seconds if seconds < HOUR => (seconds / MINUTE, "m"),
        seconds if seconds < DAY => (seconds / HOUR, "h"),
        seconds if seconds < WEEK => (seconds / DAY, "d"),
        seconds if seconds < MONTH => (seconds / WEEK, "w"),
        seconds if seconds < YEAR => (seconds / MONTH, "mo"),
        seconds => (seconds / YEAR, "y"),
    };
    format!("{amount}{unit}")
}
