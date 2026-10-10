mod actions;
mod gutter;
mod height;
mod results;

use gpui_kit::component::input::Editor;
use gpui_kit::component::text::TextView;
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::Cell;
use crate::plugins::notebook::Notebook;
use crate::plugins::notebook::cell_options::CellFont;
use crate::plugins::notebook::document::CellKind;
use actions::render_actions;
use gutter::render_gutter;
use height::{EDITOR_PADDING, editor_height};
use results::{render_outputs, render_run_status};

const CELL_GROUP: &str = "notebook-cell";

impl Notebook {
    pub(in crate::plugins::notebook) fn render_cell(
        &self,
        cell: &Cell,
        index: usize,
        window: &Window,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let id = cell.id;
        let selected = self.selected == Some(id);
        v_flex()
            .id(ElementId::named_usize("cell", id.0))
            .child(self.render_inserter(index, cx))
            .child(
                h_flex()
                    .group(CELL_GROUP)
                    .items_start()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |notebook, _, _, cx| notebook.select(id, cx)),
                    )
                    .child(render_gutter(cell, selected, cx))
                    .child(
                        v_flex()
                            .relative()
                            .flex_1()
                            .min_w_0()
                            .child(render_body(cell, &self.cell_font, selected, window, cx))
                            .children(render_run_status(cell, cx))
                            .children(render_outputs(cell, cx))
                            .child(render_actions(cell, selected, cx)),
                    ),
            )
    }
}

fn render_body(
    cell: &Cell,
    style: &CellFont,
    selected: bool,
    window: &Window,
    cx: &Context<Notebook>,
) -> AnyElement {
    if cell.kind == CellKind::Markdown && !cell.editing {
        return render_markdown(cell, selected, cx).into_any_element();
    }
    let theme = cx.theme();
    let focused = cell.editor.focus_handle(cx).is_focused(window);
    let border = if focused {
        theme.ring
    } else if selected {
        theme.border.opacity(1.8)
    } else {
        theme.border
    };
    div()
        .rounded(theme.radius)
        .border_1()
        .border_color(border)
        .bg(theme.muted)
        .overflow_hidden()
        .child(
            Editor::new(&cell.editor)
                .appearance(false)
                .readonly(cell.verbatim.is_some())
                .font_family(style.family.clone())
                .text_size(style.size)
                .line_height(relative(style.line_height))
                .px_3()
                .py(EDITOR_PADDING)
                .h(editor_height(cell, style, cx)),
        )
        .into_any_element()
}

fn render_markdown(cell: &Cell, selected: bool, cx: &Context<Notebook>) -> Stateful<Div> {
    let theme = cx.theme();
    let id = cell.id;
    let source = cell.markdown.clone();
    div()
        .id("markdown")
        .px_3()
        .py_1()
        .min_h(px(32.))
        .rounded(theme.radius)
        .border_1()
        .border_color(if selected {
            theme.border
        } else {
            theme.transparent
        })
        .on_click(
            cx.listener(move |notebook, event: &ClickEvent, window, cx| {
                if event.click_count() >= 2 {
                    notebook.edit(id, window, cx);
                }
            }),
        )
        .map(|markdown| {
            if source.trim().is_empty() {
                markdown
                    .text_sm()
                    .italic()
                    .text_color(theme.muted_foreground)
                    .child("Empty markdown cell, double-click to edit")
            } else {
                markdown.child(TextView::markdown("rendered", source))
            }
        })
}
