use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;

use gpui_kit::component::button::Button;
use gpui_kit::component::{Disableable as _, Theme, h_flex};
use gpui_kit::*;

use super::SourceControl;
use super::changes::{ChangeGroup, ChangedFile, push_tree};
use crate::plugins::git::settings::{VIEW_MODE, ViewMode};
use crate::ui::{clickable_list_row, list_row};

const ROW_HEIGHT: Pixels = px(24.);
const ROW_GROUP: &str = "source-control-row";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Section {
    Changes(ChangeGroup),
    Commits,
}

pub(super) enum Row {
    Section(Section),
    Folder(FolderRow),
    File(FileRow),
    Commit(usize),
    LoadMoreCommits,
}

pub(super) struct FolderRow {
    pub(super) group: ChangeGroup,
    pub(super) relative_path: PathBuf,
    pub(super) label: SharedString,
    pub(super) depth: usize,
    pub(super) expanded: bool,
}

pub(super) struct FileRow {
    pub(super) group: ChangeGroup,
    pub(super) file: Rc<ChangedFile>,
    pub(super) depth: usize,
    pub(super) show_folder: bool,
}

impl SourceControl {
    pub(super) fn rebuild_rows(&mut self) {
        let mut rows = Vec::new();
        for group in [ChangeGroup::Staged, ChangeGroup::Unstaged] {
            let files = self.changes.files(group);
            if group == ChangeGroup::Staged && files.is_empty() {
                continue;
            }
            let section = Section::Changes(group);
            rows.push(Row::Section(section));
            if self.collapsed_sections.contains(&section) {
                continue;
            }
            match self.view_mode {
                ViewMode::Tree => push_tree(
                    group,
                    files,
                    &self.collapsed_folders,
                    self.compact_folders,
                    &mut rows,
                ),
                ViewMode::List => rows.extend(files.iter().map(|file| {
                    Row::File(FileRow {
                        group,
                        file: Rc::clone(file),
                        depth: 0,
                        show_folder: true,
                    })
                })),
            }
        }
        if self.show_history && !self.history.entries.is_empty() {
            rows.push(Row::Section(Section::Commits));
            if !self.collapsed_sections.contains(&Section::Commits) {
                rows.extend((0..self.history.entries.len()).map(Row::Commit));
                if self.has_more_history {
                    rows.push(Row::LoadMoreCommits);
                }
            }
        }
        self.rows = rows;
    }

    pub(super) fn render_rows(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        range
            .filter_map(|index| {
                let element = match self.rows.get(index)? {
                    Row::Section(section) => self.render_section_row(index, *section, cx),
                    Row::Folder(folder) => self.render_folder_row(index, folder, cx),
                    Row::File(file) => self.render_file_row(index, file, cx),
                    Row::Commit(entry) => self.render_commit_row(index, *entry, cx),
                    Row::LoadMoreCommits => self.render_load_more_row(index, cx),
                };
                Some(element)
            })
            .collect()
    }

    pub(super) fn toggle_section(&mut self, section: Section, cx: &mut Context<Self>) {
        if !self.collapsed_sections.remove(&section) {
            self.collapsed_sections.insert(section);
        }
        self.rebuild_rows();
        cx.notify();
    }

    pub(super) fn toggle_folder(
        &mut self,
        group: ChangeGroup,
        relative_path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let key = (group, relative_path);
        if !self.collapsed_folders.remove(&key) {
            self.collapsed_folders.insert(key);
        }
        self.rebuild_rows();
        cx.notify();
    }

    pub(super) fn toggle_view_mode(&mut self, cx: &mut Context<Self>) {
        let view_mode = match self.view_mode {
            ViewMode::Tree => ViewMode::List,
            ViewMode::List => ViewMode::Tree,
        };
        VIEW_MODE.set(view_mode, cx);
        self.set_view_mode(view_mode, cx);
    }

    pub(super) fn set_view_mode(&mut self, view_mode: ViewMode, cx: &mut Context<Self>) {
        self.view_mode = view_mode;
        self.rebuild_rows();
        cx.notify();
    }
}

pub(super) fn row_container(index: usize, theme: &Theme) -> Stateful<Div> {
    list_row(("source-control-row", index), ROW_HEIGHT, theme).group(ROW_GROUP)
}

pub(super) fn clickable_row(index: usize, theme: &Theme) -> Stateful<Div> {
    clickable_list_row(("source-control-row", index), ROW_HEIGHT, theme).group(ROW_GROUP)
}

pub(super) fn indent(depth: usize) -> Pixels {
    px(24. + depth as f32 * 12.)
}

pub(super) fn hover_actions(actions: Vec<Button>, busy: bool) -> Stateful<Div> {
    h_flex()
        .id("row-actions")
        .flex_shrink_0()
        .gap(px(2.))
        .invisible()
        .group_hover(ROW_GROUP, |style| style.visible())
        .on_click(|_, _, cx| cx.stop_propagation())
        .children(actions.into_iter().map(|action| action.disabled(busy)))
}
