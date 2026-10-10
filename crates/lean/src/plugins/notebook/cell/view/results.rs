use gpui_kit::component::{ActiveTheme as _, Icon, h_flex, v_flex};
use gpui_kit::*;

use crate::plugins::notebook::Notebook;
use crate::plugins::notebook::cell::{Cell, RunState};
use crate::plugins::notebook::settings::SHOW_RUN_TIME;

pub(super) fn render_run_status(cell: &Cell, cx: &Context<Notebook>) -> Option<Div> {
    let RunState::Done { elapsed, success } = cell.run_state else {
        return None;
    };
    if !SHOW_RUN_TIME.get(cx) {
        return None;
    }
    let theme = cx.theme();
    let (icon, color) = if success {
        ("icons/check.svg", theme.success)
    } else {
        ("icons/x.svg", theme.danger)
    };
    Some(
        h_flex()
            .h(px(20.))
            .gap_1()
            .pl_0p5()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(Icon::empty().path(icon).size_3p5().text_color(color))
            .child(format!("{:.1}s", elapsed.as_secs_f32())),
    )
}

pub(super) fn render_outputs(cell: &Cell, cx: &Context<Notebook>) -> Option<Stateful<Div>> {
    if cell.outputs.is_empty() {
        return None;
    }
    Some(
        v_flex()
            .id("outputs")
            .gap_1p5()
            .pt_1p5()
            .pb_1()
            .pr_3()
            .overflow_x_scroll()
            .children(
                cell.outputs.iter().enumerate().map(|(index, output)| {
                    output.render(ElementId::named_usize("output", index), cx)
                }),
            ),
    )
}
