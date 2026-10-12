mod changes;
mod commit;
mod discard;
mod git_error;
mod header;
mod history;
mod keymap;
mod operations;
mod panel;
mod rows;
mod section_row;
mod view;

use std::collections::HashSet;
use std::path::PathBuf;

use gpui_kit::component::input::TextareaState;
use gpui_kit::*;

use super::git_project::{GitEvent, GitProject};
use crate::plugins::PanelEvent;
use crate::plugins::git::settings::{COMPACT_FOLDERS, SHOW_HISTORY, VIEW_MODE, ViewMode};
use changes::{ChangeGroup, Changes};
use history::History;
use rows::{Row, Section};

pub(crate) use keymap::bind_keys;
pub(crate) use panel::PANEL;

pub(crate) struct SourceControl {
    git: Entity<GitProject>,
    focus_handle: FocusHandle,
    commit_message: Entity<TextareaState>,
    changes: Changes,
    history: History,
    has_more_history: bool,
    rows: Vec<Row>,
    view_mode: ViewMode,
    compact_folders: bool,
    show_history: bool,
    collapsed_sections: HashSet<Section>,
    collapsed_folders: HashSet<(ChangeGroup, PathBuf)>,
    operation: Option<Task<()>>,
    scroll_handle: UniformListScrollHandle,
    _git_subscription: Subscription,
    _settings: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for SourceControl {}

impl SourceControl {
    fn new(git: Entity<GitProject>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let commit_message = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 8)
                .placeholder("Message (Ctrl+Enter to commit)")
        });
        let git_subscription = cx.subscribe_in(
            &git,
            window,
            |source_control, _, event: &GitEvent, window, cx| match event {
                GitEvent::Changed => {
                    source_control.reload(cx);
                    source_control.keep_focus_rendered(window, cx);
                }
            },
        );
        let mut source_control = Self {
            git,
            focus_handle: cx.focus_handle(),
            commit_message,
            changes: Changes::default(),
            history: History::default(),
            has_more_history: false,
            rows: Vec::new(),
            view_mode: VIEW_MODE.get(cx),
            compact_folders: COMPACT_FOLDERS.get(cx),
            show_history: SHOW_HISTORY.get(cx),
            collapsed_sections: HashSet::new(),
            collapsed_folders: HashSet::new(),
            operation: None,
            scroll_handle: UniformListScrollHandle::new(),
            _git_subscription: git_subscription,
            _settings: Self::observe_settings(cx),
        };
        source_control.reload(cx);
        source_control
    }

    fn observe_settings(cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            VIEW_MODE.observe(cx, |source_control, view_mode, cx| {
                source_control.set_view_mode(view_mode, cx)
            }),
            COMPACT_FOLDERS.observe(cx, |source_control, compact, cx| {
                source_control.compact_folders = compact;
                source_control.rebuild_rows();
                cx.notify();
            }),
            SHOW_HISTORY.observe(cx, |source_control, show, cx| {
                source_control.show_history = show;
                source_control.rebuild_rows();
                cx.notify();
            }),
        ]
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(git) = self.git.read(cx).state() {
            self.changes = Changes::collect(git);
            self.history.update(&git.history);
            self.has_more_history = git.has_more_history;
        } else {
            self.changes = Changes::default();
            self.history = History::default();
            self.has_more_history = false;
        }
        self.rebuild_rows();
        cx.notify();
    }
}

impl Focusable for SourceControl {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
