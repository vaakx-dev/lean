use gpui_kit::component::button::Button;
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::ChangeGroup;
use crate::files::file_icon;
use crate::git::FileStatus;
use crate::source_control::row_actions::{hover_actions, icon_button};
use crate::source_control::rows::{FileRow, clickable_row, indent};
use crate::source_control::{SourceControl, SourceControlEvent};

impl SourceControl {
    pub(in crate::source_control) fn render_file_row(
        &self,
        index: usize,
        row: &FileRow,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let file = &row.file;
        let is_deleted = file.status == FileStatus::Deleted;
        let path = file.path.clone();
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
            .child(hover_actions(self.file_actions(row, cx)))
            .child(
                h_flex()
                    .w_4()
                    .flex_shrink_0()
                    .justify_center()
                    .child(file.status.marker(theme)),
            )
            .when(!is_deleted, |element| {
                element.on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(SourceControlEvent::OpenFile(path.clone()))
                }))
            })
            .into_any_element()
    }

    fn file_actions(&self, row: &FileRow, cx: &Context<Self>) -> Vec<Button> {
        let mut actions = Vec::new();
        let file = row.file.clone();
        let path = file.path.clone();
        if file.status != FileStatus::Deleted {
            let path = path.clone();
            actions.push(
                icon_button("open", "icons/file-input.svg", "Open File").on_click(cx.listener(
                    move |_, _, _, cx| cx.emit(SourceControlEvent::OpenFile(path.clone())),
                )),
            );
        }
        match row.group {
            ChangeGroup::Staged => actions.push(
                icon_button("unstage", "icons/minus.svg", "Unstage Changes").on_click(cx.listener(
                    move |source_control, _, window, cx| {
                        source_control.unstage(vec![path.clone()], window, cx)
                    },
                )),
            ),
            ChangeGroup::Unstaged => {
                actions.push(
                    icon_button("discard", "icons/undo-2.svg", "Discard Changes").on_click(
                        cx.listener(move |source_control, _, window, cx| {
                            source_control.confirm_discard(vec![file.clone()], window, cx)
                        }),
                    ),
                );
                actions.push(
                    icon_button("stage", "icons/plus.svg", "Stage Changes").on_click(cx.listener(
                        move |source_control, _, window, cx| {
                            source_control.stage(vec![path.clone()], window, cx)
                        },
                    )),
                );
            }
        }
        actions
    }
}
