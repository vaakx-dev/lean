mod changes;
mod commit;
mod commit_box;
mod diff;
mod discard;
mod focus;
mod git_error;
mod header;
mod history;
mod keymap;
mod operations;
mod panel;
mod placeholder;
mod row_actions;
mod rows;
mod section_row;
mod view;

use std::collections::HashSet;
use std::path::PathBuf;

use futures::FutureExt as _;
use futures::future::Shared;
use gpui_kit::component::input::TextareaState;
use gpui_kit::*;

use super::git_project::{GitEvent, GitProject};
use crate::plugins::PanelEvent;
use changes::{ChangeGroup, Changes};
use history::History;
use rows::{Row, Section, ViewMode};

pub(crate) use keymap::bind_keys;
pub(crate) use panel::PANEL;

pub(crate) struct SourceControl {
    git: Entity<GitProject>,
    focus_handle: FocusHandle,
    commit_message: Entity<TextareaState>,
    changes: Changes,
    history: History,
    rows: Vec<Row>,
    view_mode: ViewMode,
    collapsed_sections: HashSet<Section>,
    collapsed_folders: HashSet<(ChangeGroup, PathBuf)>,
    committing: bool,
    git_operation: Shared<Task<bool>>,
    pending_git_operations: usize,
    scroll_handle: UniformListScrollHandle,
    _git_subscription: Subscription,
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
            rows: Vec::new(),
            view_mode: ViewMode::Tree,
            collapsed_sections: HashSet::new(),
            collapsed_folders: HashSet::new(),
            committing: false,
            git_operation: Task::ready(true).shared(),
            pending_git_operations: 0,
            scroll_handle: UniformListScrollHandle::new(),
            _git_subscription: git_subscription,
        };
        source_control.reload(cx);
        source_control
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(git) = self.git.read(cx).state() {
            self.changes = Changes::collect(git);
            self.history = History::build(&git.history);
        } else {
            self.changes = Changes::default();
            self.history = History::default();
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
