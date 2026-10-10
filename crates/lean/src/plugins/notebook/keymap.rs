use gpui_kit::component::input::Escape;
use gpui_kit::*;

use super::Notebook;
use super::document::CellKind;
use crate::app::keymap;

actions!(
    notebook,
    [
        RunCell,
        RunCellAndAdvance,
        RunCellAndInsert,
        RunAllCells,
        InterruptKernel,
        RestartKernel,
        SelectPreviousCell,
        SelectNextCell,
        EditCell,
        InsertCellAbove,
        InsertCellBelow,
        DeleteCell,
        ChangeCellToCode,
        ChangeCellToMarkdown,
    ]
);

pub(super) const KEY_CONTEXT: &str = "Notebook";
const EDITING_CONTEXT: &str = "Notebook > Input";
const COMMAND_CONTEXT: &str = "Notebook && !Input";

pub(crate) fn bind_keys(cx: &mut App) {
    keymap::bind_defaults(
        cx,
        [
            KeyBinding::new("shift-enter", RunCellAndAdvance, Some(EDITING_CONTEXT)),
            KeyBinding::new("ctrl-enter", RunCell, Some(EDITING_CONTEXT)),
            KeyBinding::new("alt-enter", RunCellAndInsert, Some(EDITING_CONTEXT)),
            KeyBinding::new("shift-enter", RunCellAndAdvance, Some(COMMAND_CONTEXT)),
            KeyBinding::new("ctrl-enter", RunCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("alt-enter", RunCellAndInsert, Some(COMMAND_CONTEXT)),
            KeyBinding::new("up", SelectPreviousCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("k", SelectPreviousCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("down", SelectNextCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("j", SelectNextCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("enter", EditCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("a", InsertCellAbove, Some(COMMAND_CONTEXT)),
            KeyBinding::new("b", InsertCellBelow, Some(COMMAND_CONTEXT)),
            KeyBinding::new("d d", DeleteCell, Some(COMMAND_CONTEXT)),
            KeyBinding::new("y", ChangeCellToCode, Some(COMMAND_CONTEXT)),
            KeyBinding::new("m", ChangeCellToMarkdown, Some(COMMAND_CONTEXT)),
            KeyBinding::new("i i", InterruptKernel, Some(COMMAND_CONTEXT)),
            KeyBinding::new("0 0", RestartKernel, Some(COMMAND_CONTEXT)),
        ],
    );
}

pub(super) fn bind_actions(element: Stateful<Div>, cx: &Context<Notebook>) -> Stateful<Div> {
    element
        .on_action(cx.listener(|notebook, _: &RunCell, window, cx| {
            notebook.run_selected(window, cx);
        }))
        .on_action(cx.listener(|notebook, _: &RunCellAndAdvance, window, cx| {
            if let Some(id) = notebook.run_selected(window, cx) {
                notebook.advance(id, window, cx);
            }
        }))
        .on_action(cx.listener(|notebook, _: &RunCellAndInsert, window, cx| {
            if let Some(index) = notebook.selected_index() {
                notebook.run_selected(window, cx);
                notebook.insert_cell(index + 1, CellKind::Code, window, cx);
            }
        }))
        .on_action(cx.listener(|notebook, _: &RunAllCells, _, cx| notebook.run_all(cx)))
        .on_action(
            cx.listener(|notebook, _: &InterruptKernel, _, cx| notebook.interrupt_kernel(cx)),
        )
        .on_action(cx.listener(|notebook, _: &RestartKernel, _, cx| notebook.restart_kernel(cx)))
        .on_action(
            cx.listener(|notebook, _: &SelectPreviousCell, _, cx| notebook.select_offset(-1, cx)),
        )
        .on_action(cx.listener(|notebook, _: &SelectNextCell, _, cx| notebook.select_offset(1, cx)))
        .on_action(cx.listener(|notebook, _: &EditCell, window, cx| {
            if let Some(id) = notebook.selected {
                notebook.edit(id, window, cx);
            }
        }))
        .on_action(cx.listener(|notebook, _: &InsertCellAbove, window, cx| {
            let index = notebook.selected_index().unwrap_or(0);
            notebook.insert_cell(index, CellKind::Code, window, cx);
        }))
        .on_action(cx.listener(|notebook, _: &InsertCellBelow, window, cx| {
            notebook.insert_below_selected(CellKind::Code, window, cx)
        }))
        .on_action(cx.listener(|notebook, _: &DeleteCell, window, cx| {
            if let Some(id) = notebook.selected {
                notebook.delete_cell(id, window, cx);
            }
        }))
        .on_action(cx.listener(|notebook, _: &ChangeCellToCode, _, cx| {
            if let Some(id) = notebook.selected {
                notebook.set_cell_kind(id, CellKind::Code, cx);
            }
        }))
        .on_action(cx.listener(|notebook, _: &ChangeCellToMarkdown, _, cx| {
            if let Some(id) = notebook.selected {
                notebook.set_cell_kind(id, CellKind::Markdown, cx);
            }
        }))
        .on_action(
            cx.listener(|notebook, _: &Escape, window, cx| notebook.stop_editing(window, cx)),
        )
}

impl Notebook {
    fn run_selected(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<super::cell::CellId> {
        let id = self.selected?;
        let kind = self.cell(id)?.kind;
        if kind == CellKind::Code {
            self.run_cells(vec![id], cx);
        } else {
            self.stop_editing(window, cx);
        }
        Some(id)
    }
}
