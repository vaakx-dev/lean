mod file_row;
mod line_row;
mod query_bar;
mod rows;
mod running;
mod summary;
mod view;

use std::collections::HashSet;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use super::query::SearchOptions;
use super::results::SearchResults;
use crate::plugins::{PanelEvent, PanelView};
use crate::project::Project;
use rows::Row;
use running::{SearchStatus, TYPING_DELAY};

pub(super) struct SearchPanel {
    project: Entity<Project>,
    focus_handle: FocusHandle,
    query_input: Entity<InputState>,
    include_input: Entity<InputState>,
    exclude_input: Entity<InputState>,
    options: SearchOptions,
    show_filters: bool,
    results: SearchResults,
    status: SearchStatus,
    replacing_results: bool,
    collapsed: HashSet<usize>,
    rows: Vec<Row>,
    active_line: Option<(usize, usize)>,
    scroll_handle: UniformListScrollHandle,
    search_task: Option<Task<()>>,
    _subscriptions: [Subscription; 3],
}

impl EventEmitter<PanelEvent> for SearchPanel {}

impl SearchPanel {
    pub(super) fn new(
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        let include_input = cx
            .new(|cx| InputState::new(window, cx).placeholder("Files to include (e.g. src, *.rs)"));
        let exclude_input = cx
            .new(|cx| InputState::new(window, cx).placeholder("Files to exclude (e.g. *.min.js)"));
        let subscriptions = [
            cx.subscribe(&query_input, |panel, _, event: &InputEvent, cx| {
                panel.handle_input_event(event, cx)
            }),
            cx.subscribe(&include_input, |panel, _, event: &InputEvent, cx| {
                panel.handle_input_event(event, cx)
            }),
            cx.subscribe(&exclude_input, |panel, _, event: &InputEvent, cx| {
                panel.handle_input_event(event, cx)
            }),
        ];
        Self {
            project,
            focus_handle: cx.focus_handle(),
            query_input,
            include_input,
            exclude_input,
            options: SearchOptions::default(),
            show_filters: false,
            results: SearchResults::default(),
            status: SearchStatus::Idle,
            replacing_results: false,
            collapsed: HashSet::new(),
            rows: Vec::new(),
            active_line: None,
            scroll_handle: UniformListScrollHandle::new(),
            search_task: None,
            _subscriptions: subscriptions,
        }
    }

    fn handle_input_event(&mut self, event: &InputEvent, cx: &mut Context<Self>) {
        match event {
            InputEvent::Change => self.schedule_search(TYPING_DELAY, cx),
            InputEvent::PressEnter { .. } => self.search_now(cx),
            InputEvent::Focus | InputEvent::Blur => {}
        }
    }
}

impl Focusable for SearchPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PanelView for SearchPanel {
    fn focus_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.query_input
            .update(cx, |input, cx| input.focus(window, cx));
    }
}
