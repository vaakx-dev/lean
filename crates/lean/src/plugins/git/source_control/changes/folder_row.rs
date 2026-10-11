use std::path::{Path, PathBuf};

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Icon;
use gpui_kit::component::button::Button;
use gpui_kit::*;

use super::{ChangeGroup, paths_of};
use crate::plugins::git::source_control::SourceControl;
use crate::plugins::git::source_control::rows::{FolderRow, clickable_row, hover_actions, indent};
use crate::ui::{chevron, icon_button};

impl SourceControl {
    pub(in crate::plugins::git::source_control) fn render_folder_row(
        &self,
        index: usize,
        row: &FolderRow,
        cx: &Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let group = row.group;
        let relative_path = row.relative_path.clone();
        let folder_icon = if row.expanded {
            "icons/folder-open.svg"
        } else {
            "icons/folder.svg"
        };
        clickable_row(index, theme)
            .pl(indent(row.depth) - px(20.))
            .gap_1()
            .child(chevron(row.expanded, theme))
            .child(
                Icon::empty()
                    .path(folder_icon)
                    .size_4()
                    .flex_shrink_0()
                    .text_color(theme.muted_foreground),
            )
            .child(div().flex_1().min_w_0().truncate().child(row.label.clone()))
            .child(hover_actions(self.folder_actions(row, cx), self.is_busy()))
            .on_click(cx.listener(move |source_control, _, _, cx| {
                source_control.toggle_folder(group, relative_path.clone(), cx)
            }))
            .into_any_element()
    }

    fn folder_actions(&self, row: &FolderRow, cx: &Context<Self>) -> Vec<Button> {
        let group = row.group;
        let folder = row.relative_path.clone();
        match group {
            ChangeGroup::Staged => vec![
                icon_button("unstage", "icons/minus.svg", "Unstage Changes").on_click(cx.listener(
                    move |source_control, _, window, cx| {
                        let paths = source_control.folder_paths(group, &folder);
                        source_control.unstage(paths, window, cx)
                    },
                )),
            ],
            ChangeGroup::Unstaged => {
                let discard_folder = folder.clone();
                vec![
                    icon_button("discard", "icons/undo-2.svg", "Discard Changes").on_click(
                        cx.listener(move |source_control, _, window, cx| {
                            let files = source_control
                                .changes
                                .files_in_folder(group, &discard_folder);
                            source_control.confirm_discard(files, window, cx)
                        }),
                    ),
                    icon_button("stage", "icons/plus.svg", "Stage Changes").on_click(cx.listener(
                        move |source_control, _, window, cx| {
                            let paths = source_control.folder_paths(group, &folder);
                            source_control.stage(paths, window, cx)
                        },
                    )),
                ]
            }
        }
    }

    fn folder_paths(&self, group: ChangeGroup, folder: &Path) -> Vec<PathBuf> {
        paths_of(&self.changes.files_in_folder(group, folder))
    }
}
