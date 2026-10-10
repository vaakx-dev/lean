use gpui_kit::*;

use super::Notebook;
use super::cell_options::cell_tab_size;
use super::settings::{INTERPRETER, LINE_NUMBERS, OUTPUT_MAX_LINES, SHOW_RUN_TIME, SOFT_WRAP};
use crate::editor::settings::{FONT_FAMILY, FONT_SIZE, INSERT_SPACES, LINE_HEIGHT, TAB_SIZE};

impl Notebook {
    pub(super) fn observe_settings(window: &Window, cx: &mut Context<Self>) -> Vec<Subscription> {
        vec![
            TAB_SIZE.observe(cx, |notebook, _, cx| notebook.apply_tab_size(cx)),
            INSERT_SPACES.observe(cx, |notebook, _, cx| notebook.apply_tab_size(cx)),
            FONT_FAMILY.observe(cx, |notebook, _, cx| notebook.relayout(cx)),
            FONT_SIZE.observe(cx, |notebook, _, cx| notebook.relayout(cx)),
            LINE_HEIGHT.observe(cx, |notebook, _, cx| notebook.relayout(cx)),
            LINE_NUMBERS.observe_in(window, cx, |notebook, enabled, window, cx| {
                for cell in &notebook.cells {
                    cell.editor
                        .update(cx, |editor, cx| editor.set_line_number(enabled, window, cx));
                }
                notebook.relayout(cx);
            }),
            SOFT_WRAP.observe_in(window, cx, |notebook, enabled, window, cx| {
                for cell in &notebook.cells {
                    cell.editor
                        .update(cx, |editor, cx| editor.set_soft_wrap(enabled, window, cx));
                }
                notebook.relayout(cx);
            }),
            OUTPUT_MAX_LINES.observe(cx, |notebook, _, cx| {
                for cell in &mut notebook.cells {
                    cell.rebuild_outputs(cx);
                }
                notebook.relayout(cx);
            }),
            SHOW_RUN_TIME.observe(cx, |notebook, _, cx| notebook.relayout(cx)),
            INTERPRETER.observe(cx, |notebook, _, cx| notebook.refresh_outdated(cx)),
        ]
    }

    fn apply_tab_size(&mut self, cx: &mut Context<Self>) {
        let tab_size = cell_tab_size(cx);
        for cell in &self.cells {
            cell.editor
                .update(cx, |editor, cx| editor.set_tab_size(tab_size, cx));
        }
    }

    fn relayout(&mut self, cx: &mut Context<Self>) {
        self.list_state.remeasure();
        cx.notify();
    }
}
