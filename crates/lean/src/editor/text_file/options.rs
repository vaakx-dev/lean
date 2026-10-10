use gpui_kit::component::input::{EditorState, TabSize};
use gpui_kit::*;

use crate::editor::settings::{
    AUTO_CLOSE_BRACKETS, CODE_FOLDING, INDENT_GUIDES, INSERT_SPACES, LINE_NUMBERS, SHOW_WHITESPACE,
    SMART_INDENT, SOFT_WRAP, tab_size,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct EditorOptions {
    tab_size: usize,
    hard_tabs: bool,
    smart_indent: bool,
    auto_close: bool,
    line_numbers: bool,
    indent_guides: bool,
    folding: bool,
    soft_wrap: bool,
    show_whitespace: bool,
}

impl EditorOptions {
    pub(super) fn read(cx: &App) -> Self {
        Self {
            tab_size: tab_size(cx),
            hard_tabs: !INSERT_SPACES.get(cx),
            smart_indent: SMART_INDENT.get(cx),
            auto_close: AUTO_CLOSE_BRACKETS.get(cx),
            line_numbers: LINE_NUMBERS.get(cx),
            indent_guides: INDENT_GUIDES.get(cx),
            folding: CODE_FOLDING.get(cx),
            soft_wrap: SOFT_WRAP.get(cx),
            show_whitespace: SHOW_WHITESPACE.get(cx),
        }
    }

    pub(super) fn indent_label(&self) -> String {
        if self.hard_tabs {
            format!("Tab Size: {}", self.tab_size)
        } else {
            format!("Spaces: {}", self.tab_size)
        }
    }

    pub(super) fn configure(&self, editor: EditorState) -> EditorState {
        editor
            .tab_size(self.tab())
            .smart_indent(self.smart_indent)
            .auto_close(self.auto_close)
            .line_number(self.line_numbers)
            .indent_guides(self.indent_guides)
            .folding(self.folding)
            .soft_wrap(self.soft_wrap)
            .show_whitespaces(self.show_whitespace)
    }

    pub(super) fn apply(
        &self,
        previous: &Self,
        editor: &mut EditorState,
        window: &mut Window,
        cx: &mut Context<EditorState>,
    ) {
        if self.tab_size != previous.tab_size || self.hard_tabs != previous.hard_tabs {
            editor.set_tab_size(self.tab(), cx);
        }
        if self.smart_indent != previous.smart_indent {
            editor.set_smart_indent(self.smart_indent, window, cx);
        }
        if self.auto_close != previous.auto_close {
            editor.set_auto_close(self.auto_close, window, cx);
        }
        if self.line_numbers != previous.line_numbers {
            editor.set_line_number(self.line_numbers, window, cx);
        }
        if self.indent_guides != previous.indent_guides {
            editor.set_indent_guides(self.indent_guides, window, cx);
        }
        if self.folding != previous.folding {
            editor.set_folding(self.folding, window, cx);
        }
        if self.soft_wrap != previous.soft_wrap {
            editor.set_soft_wrap(self.soft_wrap, window, cx);
        }
        if self.show_whitespace != previous.show_whitespace {
            editor.set_show_whitespaces(self.show_whitespace, window, cx);
        }
    }

    fn tab(&self) -> TabSize {
        TabSize {
            tab_size: self.tab_size,
            hard_tabs: self.hard_tabs,
        }
    }
}
