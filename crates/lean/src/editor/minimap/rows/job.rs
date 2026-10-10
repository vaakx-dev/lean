use std::ops::Range;
use std::sync::Arc;

use gpui_kit::component::input::{
    EditorState, InputHighlightSnapshot, Rope, RopeExt as _, SharedHighlightStyleResolver,
};
use gpui_kit::*;

use super::super::geometry::MAX_COLUMNS;
use super::super::style::MinimapStyle;
use super::style_cursor::StyleCursor;
use super::{Rows, RowsBuilder};

const MAX_LINE_BYTES: usize = 10_000;

pub(in super::super) struct RowsJob {
    ranges: Vec<Range<usize>>,
    source: Source,
    foreground: Hsla,
    tab_columns: usize,
}

enum Source {
    Plain(Rope),
    Highlighted {
        snapshot: Arc<dyn InputHighlightSnapshot>,
        resolver: SharedHighlightStyleResolver,
    },
}

impl Source {
    fn text(&self) -> &Rope {
        match self {
            Self::Plain(text) => text,
            Self::Highlighted { snapshot, .. } => snapshot.text(),
        }
    }

    fn styles(&self, range: Range<usize>) -> Vec<(Range<usize>, HighlightStyle)> {
        match self {
            Self::Plain(_) => Vec::new(),
            Self::Highlighted { snapshot, resolver } => snapshot.styles(&range, resolver.as_ref()),
        }
    }
}

impl RowsJob {
    pub(in super::super) fn new(
        editor: &EditorState,
        ranges: Vec<Range<usize>>,
        foreground: Hsla,
        style: &MinimapStyle,
    ) -> Self {
        let snapshot = editor.highlight_snapshot().filter(|_| style.syntax_colors);
        let source = match snapshot {
            Some(snapshot) => Source::Highlighted {
                snapshot,
                resolver: editor.highlight_style_resolver(),
            },
            None => Source::Plain(editor.text().clone()),
        };
        Self {
            ranges,
            source,
            foreground,
            tab_columns: style.tab_columns.max(1),
        }
    }

    pub(in super::super) fn run(self) -> Vec<Rows> {
        self.ranges
            .iter()
            .map(|lines| self.build(lines.clone()))
            .collect()
    }

    fn build(&self, lines: Range<usize>) -> Rows {
        let text = self.source.text();
        let end = lines.end.min(text.lines_len());
        let mut rows = RowsBuilder::new(lines.start);
        let mut line = lines.start;
        while line < end {
            let run_end = (line..end)
                .find(|&line| text.line_len(line) > MAX_LINE_BYTES)
                .unwrap_or(end);
            if run_end == line {
                self.push_line(line, &mut StyleCursor::new(&[], self.foreground), &mut rows);
                line += 1;
                continue;
            }
            let range = text.line_start_offset(line)..text.line_end_offset(run_end - 1);
            let styles = self.source.styles(range);
            let mut colors = StyleCursor::new(&styles, self.foreground);
            for line in line..run_end {
                self.push_line(line, &mut colors, &mut rows);
            }
            line = run_end;
        }
        rows.finish()
    }

    fn push_line(&self, line: usize, colors: &mut StyleCursor<'_>, rows: &mut RowsBuilder) {
        let text = self.source.text();
        let tab = self.tab_columns;
        let mut offset = text.line_start_offset(line);
        let mut column = 0;
        for character in text.slice_line(line).chars() {
            if column >= MAX_COLUMNS {
                break;
            }
            if character == '\t' {
                column += tab - column % tab;
            } else if character.is_whitespace() {
                column += 1;
            } else {
                rows.push(column, colors.color_at(offset));
                column += 1;
            }
            offset += character.len_utf8();
        }
        rows.end_row();
    }
}
