mod changes;
mod commit;
mod commit_box;
mod discard;
mod git_error;
mod header;
mod history;
mod keymap;
mod operations;
mod row_actions;
mod rows;
mod section_row;
mod view;

use std::collections::HashSet;
use std::path::PathBuf;

use gpui_kit::component::input::TextareaState;
use gpui_kit::*;

use crate::project::{Project, ProjectEvent};
use changes::{ChangeGroup, Changes};
use history::History;
use rows::{Row, Section, ViewMode};

pub(crate) use keymap::bind_keys;

pub(crate) enum SourceControlEvent {
    OpenFile(PathBuf),
}

pub(crate) struct SourceControl {
    project: Entity<Project>,
    commit_message: Entity<TextareaState>,
    changes: Changes,
    history: History,
    rows: Vec<Row>,
    view_mode: ViewMode,
    collapsed_sections: HashSet<Section>,
    collapsed_folders: HashSet<(ChangeGroup, PathBuf)>,
    committing: bool,
    scroll_handle: UniformListScrollHandle,
    _project_subscription: Subscription,
}

impl EventEmitter<SourceControlEvent> for SourceControl {}

impl SourceControl {
    pub(crate) fn new(
        project: Entity<Project>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let commit_message = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 8)
                .placeholder("Message (Ctrl+Enter to commit)")
        });
        let project_subscription = cx.subscribe(
            &project,
            |source_control, _, event: &ProjectEvent, cx| match event {
                ProjectEvent::GitChanged => source_control.reload(cx),
            },
        );
        let mut source_control = Self {
            project,
            commit_message,
            changes: Changes::default(),
            history: History::default(),
            rows: Vec::new(),
            view_mode: ViewMode::Tree,
            collapsed_sections: HashSet::new(),
            collapsed_folders: HashSet::new(),
            committing: false,
            scroll_handle: UniformListScrollHandle::new(),
            _project_subscription: project_subscription,
        };
        source_control.reload(cx);
        source_control
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(git) = self.project.read(cx).git() {
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
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.commit_message.focus_handle(cx)
    }
}
