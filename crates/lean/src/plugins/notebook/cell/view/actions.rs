use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::CELL_GROUP;
use crate::plugins::notebook::Notebook;
use crate::plugins::notebook::cell::Cell;
use crate::plugins::notebook::document::CellKind;
use crate::ui::icon_button;

pub(super) fn render_actions(cell: &Cell, selected: bool, cx: &Context<Notebook>) -> Div {
    let theme = cx.theme();
    let id = cell.id;
    let unsupported = cell.verbatim.is_some();
    let (kind_label, next_kind) = match cell.kind {
        _ if unsupported => ("Unsupported", CellKind::Raw),
        CellKind::Code => ("Python", CellKind::Markdown),
        CellKind::Markdown => ("Markdown", CellKind::Code),
        CellKind::Raw => ("Raw", CellKind::Code),
    };
    h_flex()
        .absolute()
        .top(px(-12.))
        .right(px(6.))
        .gap(px(1.))
        .p(px(2.))
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.popover)
        .when(!selected, |actions| {
            actions
                .invisible()
                .group_hover(CELL_GROUP, |style| style.visible())
        })
        .map(|actions| match cell.kind {
            CellKind::Code => actions.child(
                icon_button("run-cell", "icons/play.svg", "Run Cell").on_click(
                    cx.listener(move |notebook, _, _, cx| notebook.run_cells(vec![id], cx)),
                ),
            ),
            CellKind::Markdown | CellKind::Raw => actions.child(
                icon_button("edit-cell", "icons/pencil.svg", "Edit Cell").on_click(
                    cx.listener(move |notebook, _, window, cx| notebook.edit(id, window, cx)),
                ),
            ),
        })
        .child(
            icon_button("move-up", "icons/arrow-up.svg", "Move Cell Up")
                .on_click(cx.listener(move |notebook, _, _, cx| notebook.move_cell(id, -1, cx))),
        )
        .child(
            icon_button("move-down", "icons/arrow-down.svg", "Move Cell Down")
                .on_click(cx.listener(move |notebook, _, _, cx| notebook.move_cell(id, 1, cx))),
        )
        .child(
            Button::new("cell-kind")
                .ghost()
                .xsmall()
                .label(kind_label)
                .tooltip(if unsupported {
                    "Unknown cell type, saved unchanged"
                } else {
                    "Change Cell Type"
                })
                .disabled(unsupported)
                .on_click(
                    cx.listener(move |notebook, _, _, cx| {
                        notebook.set_cell_kind(id, next_kind, cx)
                    }),
                ),
        )
        .when(cell.kind == CellKind::Code, |actions| {
            actions.child(
                icon_button("clear-output", "icons/eraser.svg", "Clear Output").on_click(
                    cx.listener(move |notebook, _, _, cx| notebook.clear_cell_outputs(id, cx)),
                ),
            )
        })
        .child(
            icon_button("delete-cell", "icons/trash.svg", "Delete Cell").on_click(
                cx.listener(move |notebook, _, window, cx| notebook.delete_cell(id, window, cx)),
            ),
        )
}
