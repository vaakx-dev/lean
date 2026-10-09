use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SearchPanel;
use super::rows::clickable_row;
use crate::plugins::PanelEvent;

const LINE_INDENT: Pixels = px(42.);

impl SearchPanel {
    pub(super) fn render_line_row(
        &self,
        index: usize,
        file_index: usize,
        line_index: usize,
        cx: &Context<Self>,
    ) -> Option<AnyElement> {
        let theme = cx.theme();
        let line = self.results.files.get(file_index)?.lines.get(line_index)?;
        let active = self.active_line == Some((file_index, line_index));
        let highlight = HighlightStyle {
            background_color: Some(theme.yellow.opacity(0.35)),
            ..HighlightStyle::default()
        };
        let highlights: Vec<_> = line
            .highlights
            .iter()
            .map(|range| (range.clone(), highlight))
            .collect();
        let row = clickable_row(index, theme)
            .pl(LINE_INDENT)
            .when(active, |row| row.bg(theme.list_active))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .child(StyledText::new(line.preview.clone()).with_highlights(highlights)),
            )
            .on_click(
                cx.listener(move |panel, _, _, cx| panel.open_match(file_index, line_index, cx)),
            );
        Some(row.into_any_element())
    }

    fn open_match(&mut self, file_index: usize, line_index: usize, cx: &mut Context<Self>) {
        let Some(file) = self.results.files.get(file_index) else {
            return;
        };
        let Some(line) = file.lines.get(line_index) else {
            return;
        };
        let event = PanelEvent::OpenFileAt {
            path: file.path.clone(),
            line: line.line,
            column: line.column,
        };
        self.active_line = Some((file_index, line_index));
        cx.emit(event);
        cx.notify();
    }
}
