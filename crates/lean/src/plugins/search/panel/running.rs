use std::time::Duration;

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui_kit::{Context, ScrollStrategy, SharedString};

use super::SearchPanel;
use crate::plugins::search::engine;
use crate::plugins::search::query::{SearchOptions, SearchQuery};
use crate::plugins::search::results::{SearchBatch, SearchResults};

pub(super) enum SearchStatus {
    Idle,
    Searching,
    Done,
    Failed(SharedString),
}

impl SearchPanel {
    pub(super) fn search_now(&mut self, cx: &mut Context<Self>) {
        self.schedule_search(Duration::ZERO, cx);
    }

    pub(super) fn update_options(
        &mut self,
        change: impl FnOnce(&mut SearchOptions),
        cx: &mut Context<Self>,
    ) {
        change(&mut self.options);
        self.search_now(cx);
    }

    pub(super) fn toggle_filters(&mut self, cx: &mut Context<Self>) {
        self.show_filters = !self.show_filters;
        let has_filters = !self.include_input.read(cx).value().trim().is_empty()
            || !self.exclude_input.read(cx).value().trim().is_empty();
        if has_filters {
            self.search_now(cx);
        } else {
            cx.notify();
        }
    }

    pub(super) fn schedule_search(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let query = self.query(cx);
        if query.pattern.is_empty() {
            self.clear_results(cx);
            return;
        }
        let root = self.project.read(cx).root().to_path_buf();
        let job = match engine::prepare(root, query) {
            Ok(job) => job,
            Err(message) => {
                self.clear_results(cx);
                self.status = SearchStatus::Failed(message);
                return;
            }
        };
        self.status = SearchStatus::Searching;
        self.replacing_results = true;
        cx.notify();
        self.search_task = Some(cx.spawn(async move |panel, cx| {
            cx.background_executor().timer(delay).await;
            let (sender, mut batches) = mpsc::unbounded();
            cx.background_executor()
                .spawn(async move { engine::run(job, sender) })
                .detach();
            while let Some(batch) = batches.next().await {
                if panel
                    .update(cx, |panel, cx| panel.receive_batch(batch, cx))
                    .is_err()
                {
                    return;
                }
            }
            panel.update(cx, |panel, cx| panel.finish_search(cx)).ok();
        }));
    }

    pub(super) fn clear_results(&mut self, cx: &mut Context<Self>) {
        self.search_task = None;
        self.status = SearchStatus::Idle;
        self.replacing_results = false;
        self.reset_results();
        cx.notify();
    }

    fn query(&self, cx: &Context<Self>) -> SearchQuery {
        let (include, exclude) = if self.show_filters {
            (
                self.include_input.read(cx).value().to_string(),
                self.exclude_input.read(cx).value().to_string(),
            )
        } else {
            (String::new(), String::new())
        };
        SearchQuery {
            pattern: self.query_input.read(cx).value().to_string(),
            options: self.options,
            include,
            exclude,
            config: self.config.clone(),
        }
    }

    fn receive_batch(&mut self, batch: SearchBatch, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.replacing_results) {
            self.reset_results();
        }
        self.results.append(batch);
        self.rebuild_rows();
        cx.notify();
    }

    fn finish_search(&mut self, cx: &mut Context<Self>) {
        self.status = SearchStatus::Done;
        cx.notify();
    }

    fn reset_results(&mut self) {
        self.results = SearchResults::default();
        self.collapsed.clear();
        self.active_line = None;
        self.rebuild_rows();
        self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
    }
}
