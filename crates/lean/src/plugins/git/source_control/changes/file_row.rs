use std::rc::Rc;

use gpui_kit::component::button::Button;
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::{ChangeGroup, ChangedFile};
use crate::files::file_icon;
use crate::plugins::PanelEvent;
use crate::plugins::git::diff::DiffBase;
use crate::plugins::git::repo::FileStatus;
use crate::plugins::git::source_control::SourceControl;
use crate::plugins::git::source_control::rows::{FileRow, clickable_row, hover_actions, indent};
use crate::ui::icon_button;

impl SourceControl {
    pub(in crate::plugins::git::source_control) fn render_file_row(
        &self,
        index: usize,
        row: &FileRow,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let file = &row.file;
        let is_deleted = file.status == FileStatus::Deleted;
        let clicked = Rc::clone(file);
        let group = row.group;
        clickable_row(index, theme)
            .pl(indent(row.depth))
            .child(file_icon(&file.name).flex_shrink_0())
            .child(
                div()
                    .flex_shrink_1()
                    .min_w_0()
                    .truncate()
                    .when(is_deleted, |name| name.line_through())
                    .child(file.name.clone()),
            )
            .when(row.show_folder && !file.folder.is_empty(), |element| {
                element.child(
                    div()
                        .flex_shrink_1()
                        .min_w(px(24.))
                        .truncate()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(file.folder.clone()),
                )
            })
            .child(div().flex_1())
            .child(hover_actions(self.file_actions(row, cx), self.is_busy()))
            .child(
                h_flex()
                    .w_4()
                    .flex_shrink_0()
                    .justify_center()
                    .child(file.status.marker(theme)),
            )
            .on_click(cx.listener(move |source_control, _, window, cx| {
                source_control.open_diff(&clicked, group, window, cx)
            }))
            .into_any_element()
    }

    fn file_actions(&self, row: &FileRow, cx: &Context<Self>) -> Vec<Button> {
        let mut actions = Vec::new();
        if row.file.status != FileStatus::Deleted {
            let file = Rc::clone(&row.file);
            actions.push(
                icon_button("open", "icons/file-input.svg", "Open File").on_click(
                    cx.listener(move |_, _, _, cx| {
                        cx.emit(PanelEvent::OpenFile(file.path.clone()))
                    }),
                ),
            );
        }
        match row.group {
            ChangeGroup::Staged => {
                let file = Rc::clone(&row.file);
                actions.push(
                    icon_button("unstage", "icons/minus.svg", "Unstage Changes").on_click(
                        cx.listener(move |source_control, _, window, cx| {
                            source_control.unstage(vec![file.path.clone()], window, cx)
                        }),
                    ),
                );
            }
            ChangeGroup::Unstaged => {
                let file = Rc::clone(&row.file);
                actions.push(
                    icon_button("discard", "icons/undo-2.svg", "Discard Changes").on_click(
                        cx.listener(move |source_control, _, window, cx| {
                            source_control.confirm_discard(vec![Rc::clone(&file)], window, cx)
                        }),
                    ),
                );
                let file = Rc::clone(&row.file);
                actions.push(
                    icon_button("stage", "icons/plus.svg", "Stage Changes").on_click(cx.listener(
                        move |source_control, _, window, cx| {
                            source_control.stage(vec![file.path.clone()], window, cx)
                        },
                    )),
                );
            }
        }
        actions
    }

    fn open_diff(
        &mut self,
        file: &ChangedFile,
        group: ChangeGroup,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base = match group {
            ChangeGroup::Staged => DiffBase::Staged,
            ChangeGroup::Unstaged => DiffBase::Index,
        };
        let path = file.path.clone();
        self.git
            .update(cx, |git, cx| git.open_diff(path, base, window, cx));
    }
}
