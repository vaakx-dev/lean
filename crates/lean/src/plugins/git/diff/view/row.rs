use std::ops::Range;

use gpui_kit::component::{ActiveTheme as _, Theme, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::DiffView;
use super::document::{DiffDocument, DiffRow, RowKind};

const ROW_HEIGHT: Pixels = px(20.);
const LINE_TINT: f32 = 0.12;
const NUMBER_TINT: f32 = 0.2;

impl DiffView {
    pub(super) fn render_rows(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let Some(document) = &self.document else {
            return Vec::new();
        };
        let theme = cx.theme();
        let number_width = document.number_width();
        range
            .filter_map(|index| {
                let element = match document.rows.get(index)? {
                    DiffRow::Header(header) => render_header(index, header.clone(), theme),
                    DiffRow::Line {
                        kind,
                        old_line,
                        new_line,
                    } => render_line(
                        index,
                        document,
                        *kind,
                        [*old_line, *new_line],
                        number_width,
                        theme,
                    ),
                };
                Some(element)
            })
            .collect()
    }
}

fn render_header(index: usize, header: SharedString, theme: &Theme) -> AnyElement {
    h_flex()
        .id(index)
        .h(ROW_HEIGHT)
        .px_2()
        .whitespace_nowrap()
        .overflow_hidden()
        .bg(theme.muted)
        .text_color(theme.muted_foreground)
        .child(header)
        .into_any_element()
}

fn render_line(
    index: usize,
    document: &DiffDocument,
    kind: RowKind,
    numbers: [Option<usize>; 2],
    number_width: usize,
    theme: &Theme,
) -> AnyElement {
    let [old_line, new_line] = numbers;
    let source = match kind {
        RowKind::Removed => old_line.map(|line| (&document.old, line)),
        RowKind::Added | RowKind::Context => new_line.map(|line| (&document.new, line)),
    };
    let line = source.map(|(side, line)| side.line(line, &*theme.highlight_theme));
    let (sign, tint) = match kind {
        RowKind::Context => (" ", None),
        RowKind::Added => ("+", Some(theme.success)),
        RowKind::Removed => ("-", Some(theme.danger)),
    };
    let text = match line {
        Some(line) => StyledText::new(line.text).with_highlights(line.styles),
        None => StyledText::new(""),
    };
    h_flex()
        .id(index)
        .h(ROW_HEIGHT)
        .whitespace_nowrap()
        .overflow_hidden()
        .when_some(tint, |row, tint| row.bg(tint.opacity(LINE_TINT)))
        .child(render_number(old_line, number_width, tint, theme))
        .child(render_number(new_line, number_width, tint, theme))
        .child(
            div()
                .flex_shrink_0()
                .px_1()
                .text_color(tint.unwrap_or(theme.muted_foreground))
                .child(sign),
        )
        .child(div().flex_1().text_color(theme.foreground).child(text))
        .into_any_element()
}

fn render_number(line: Option<usize>, width: usize, tint: Option<Hsla>, theme: &Theme) -> Div {
    let label = match line {
        Some(line) => format!("{:>width$}", line + 1),
        None => " ".repeat(width),
    };
    div()
        .flex_shrink_0()
        .h_full()
        .flex()
        .items_center()
        .px_2()
        .text_color(theme.muted_foreground)
        .when_some(tint, |number, tint| number.bg(tint.opacity(NUMBER_TINT)))
        .child(label)
}
