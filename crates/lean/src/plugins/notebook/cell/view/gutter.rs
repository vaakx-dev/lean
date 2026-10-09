use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::CELL_GROUP;
use crate::plugins::notebook::Notebook;
use crate::plugins::notebook::cell::{Cell, RunState};
use crate::plugins::notebook::document::CellKind;

const GUTTER_WIDTH: Pixels = px(52.);

pub(super) fn render_gutter(cell: &Cell, selected: bool, cx: &Context<Notebook>) -> Div {
    let theme = cx.theme();
    v_flex()
        .relative()
        .w(GUTTER_WIDTH)
        .flex_shrink_0()
        .self_stretch()
        .items_center()
        .pt_1()
        .gap_0p5()
        .when(selected, |gutter| {
            gutter.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px(6.))
                    .w(px(2.))
                    .rounded_full()
                    .bg(theme.primary),
            )
        })
        .when(cell.kind == CellKind::Code, |gutter| {
            gutter.child(run_button(cell, selected, cx)).child(
                div()
                    .font_family(theme.mono_font_family.clone())
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(execution_label(cell)),
            )
        })
}

fn run_button(cell: &Cell, selected: bool, cx: &Context<Notebook>) -> Div {
    let id = cell.id;
    let (icon, tooltip) = match cell.run_state {
        RunState::Queued => ("icons/clock.svg", "Queued"),
        RunState::Running => ("icons/square.svg", "Interrupt (I I)"),
        RunState::Idle | RunState::Done { .. } => ("icons/play.svg", "Run Cell (Ctrl+Enter)"),
    };
    let running = matches!(cell.run_state, RunState::Running);
    div()
        .when(!selected && !cell.is_pending(), |wrapper| {
            wrapper
                .invisible()
                .group_hover(CELL_GROUP, |style| style.visible())
        })
        .child(
            Button::new("run")
                .ghost()
                .xsmall()
                .icon(Icon::empty().path(icon))
                .tooltip(tooltip)
                .on_click(cx.listener(move |notebook, _, _, cx| {
                    if running {
                        notebook.interrupt_kernel(cx);
                    } else {
                        notebook.run_cells(vec![id], cx);
                    }
                })),
        )
}

fn execution_label(cell: &Cell) -> SharedString {
    match (&cell.run_state, cell.execution_count) {
        (RunState::Queued | RunState::Running, _) => "[*]".into(),
        (_, Some(count)) => format!("[{count}]").into(),
        (_, None) => "[ ]".into(),
    }
}
