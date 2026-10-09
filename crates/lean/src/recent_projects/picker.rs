use std::path::{Path, PathBuf};
use std::rc::Rc;

use gpui_kit::component::command::{Command, CommandItem, CommandState};
use gpui_kit::component::{ActiveTheme as _, Icon, IndexPath, WindowExt as _, h_flex};
use gpui_kit::*;

use super::{display_name, display_path, others};

const DIALOG_WIDTH: Pixels = px(560.);

pub(crate) fn open_picker(
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(PathBuf, &mut Window, &mut App) + 'static,
) {
    let paths = Rc::new(others(cx));
    let items: Vec<CommandItem> = paths.iter().map(|path| project_item(path)).collect();
    let state = cx.new(|cx| CommandState::new(window, cx));
    let on_pick = Rc::new(on_pick);
    let command_state = state.clone();
    window.open_dialog(cx, move |dialog, _, _| {
        let paths = paths.clone();
        let on_pick = on_pick.clone();
        dialog.width(DIALOG_WIDTH).p_0().close_button(false).child(
            Command::new(&command_state)
                .bordered(false)
                .placeholder("Open a recent folder")
                .items(items.clone())
                .empty(|_, _, cx| {
                    div()
                        .py_6()
                        .w_full()
                        .text_center()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("No recent folders")
                })
                .on_confirm(move |index: IndexPath, window, cx| {
                    let Some(path) = paths.get(index.row).cloned() else {
                        return;
                    };
                    window.close_dialog(cx);
                    on_pick(path, window, cx);
                }),
        )
    });
    state.update(cx, |state, cx| state.focus(window, cx));
}

fn project_item(path: &Path) -> CommandItem {
    let name: SharedString = display_name(path).into();
    let location: SharedString = display_path(path).into();
    CommandItem::new()
        .label(name.clone())
        .keywords([location.clone()])
        .child(move |_, cx| {
            h_flex()
                .flex_1()
                .min_w_0()
                .gap_2()
                .child(
                    Icon::empty()
                        .path("icons/folder.svg")
                        .size_4()
                        .flex_shrink_0()
                        .text_color(cx.theme().muted_foreground),
                )
                .child(div().flex_shrink_0().child(name.clone()))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(location.clone()),
                )
        })
}
