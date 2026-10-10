mod line_prompt;

use std::path::PathBuf;

use gpui_kit::*;

use super::EditorArea;

#[derive(Clone, Copy)]
pub(super) struct GoTo {
    pub(super) line: usize,
    pub(super) column: usize,
}

impl EditorArea {
    pub(crate) fn open_at(
        &mut self,
        path: PathBuf,
        line: usize,
        column: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_with(path, Some(GoTo { line, column }), window, cx);
    }

    pub(super) fn go_to(
        &mut self,
        index: usize,
        target: GoTo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(file) = self.files.get(index) else {
            return;
        };
        file.view.go_to(target.line, target.column, window, cx);
    }
}
