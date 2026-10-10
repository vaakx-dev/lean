use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::capture::{TableState, Target};
use super::keys::{escape_chip, keystroke_chips, sequence_chips};
use crate::app::keymap::{Row, remove_keys, reset_keys};

pub(super) const KEYS_WIDTH: Pixels = px(210.);
pub(super) const CONTEXT_WIDTH: Pixels = px(110.);
pub(super) const SOURCE_WIDTH: Pixels = px(60.);
pub(super) const ACTIONS_WIDTH: Pixels = px(52.);
const ROW_HEIGHT: Pixels = px(34.);

pub(super) fn render_row(
    row: &Row,
    conflicts: &[SharedString],
    state: &Entity<TableState>,
    cx: &App,
) -> Div {
    let theme = cx.theme();
    let target = Target::Command(row.id.clone());
    let recording = state.read(cx).is_recording(&target);
    h_flex()
        .w_full()
        .min_h(ROW_HEIGHT)
        .px_3()
        .py_1()
        .gap_3()
        .border_t_1()
        .border_color(theme.table_row_border)
        .when(recording, |line| line.bg(theme.list_active))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_sm()
                .child(row.title.clone()),
        )
        .child(div().w(KEYS_WIDTH).flex_shrink_0().child(if recording {
            capture_field(state, cx).into_any_element()
        } else {
            keys_cell(row, conflicts, target, state, cx).into_any_element()
        }))
        .child(
            div()
                .w(CONTEXT_WIDTH)
                .flex_shrink_0()
                .truncate()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(row.context.clone()),
        )
        .child(
            div()
                .w(SOURCE_WIDTH)
                .flex_shrink_0()
                .text_sm()
                .text_color(if row.customized {
                    theme.primary
                } else {
                    theme.muted_foreground
                })
                .child(if row.customized { "User" } else { "Default" }),
        )
        .child(row_actions(row))
}

fn keys_cell(
    row: &Row,
    conflicts: &[SharedString],
    target: Target,
    state: &Entity<TableState>,
    cx: &App,
) -> Stateful<Div> {
    let theme = cx.theme();
    let state = state.clone();
    let warning = if !row.available {
        Some(SharedString::from("Unknown command"))
    } else if conflicts.is_empty() {
        None
    } else {
        Some(SharedString::from(format!(
            "Also bound to {}",
            conflicts.join(", ")
        )))
    };
    let keys = if row.keys.is_empty() {
        div()
            .text_sm()
            .text_color(theme.muted_foreground)
            .child("Unbound")
    } else {
        v_flex()
            .gap_1()
            .children(row.keys.iter().map(|sequence| sequence_chips(sequence, cx)))
    };
    h_flex()
        .id(SharedString::from(format!("keys-{}", row.id)))
        .w_full()
        .gap_1p5()
        .items_start()
        .px_1()
        .py_0p5()
        .rounded(theme.radius)
        .cursor_pointer()
        .hover(|cell| cell.bg(theme.muted))
        .when(warning.is_some(), |cell| {
            cell.child(
                Icon::empty()
                    .path("icons/triangle-alert.svg")
                    .size_3p5()
                    .mt_0p5()
                    .flex_shrink_0()
                    .text_color(theme.warning),
            )
        })
        .child(
            v_flex()
                .min_w_0()
                .gap_1()
                .child(keys)
                .when_some(warning, |cell, warning| {
                    cell.child(div().text_xs().text_color(theme.warning).child(warning))
                }),
        )
        .on_click(move |_, window, cx| {
            state.update(cx, |state, cx| state.toggle(target.clone(), window, cx));
        })
}

fn capture_field(state: &Entity<TableState>, cx: &App) -> Div {
    let theme = cx.theme();
    let table = state.read(cx);
    let keystrokes = table
        .recording
        .as_ref()
        .map(|recording| recording.keystrokes.as_slice())
        .unwrap_or_default();
    h_flex()
        .track_focus(&table.focus_handle)
        .w_full()
        .h(px(26.))
        .px_2()
        .gap_2()
        .justify_between()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.ring)
        .bg(theme.background)
        .child(if keystrokes.is_empty() {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Press keys…")
        } else {
            keystroke_chips(keystrokes)
        })
        .child(escape_chip())
}

fn row_actions(row: &Row) -> Div {
    let remove_id = row.id.clone();
    let reset_id = row.id.clone();
    h_flex()
        .w(ACTIONS_WIDTH)
        .flex_shrink_0()
        .justify_end()
        .gap_0p5()
        .when(!row.keys.is_empty() && row.available, |actions| {
            actions.child(
                Button::new(SharedString::from(format!("remove-{}", row.id)))
                    .ghost()
                    .xsmall()
                    .icon(Icon::empty().path("icons/x.svg"))
                    .tooltip("Remove keybinding")
                    .on_click(move |_, _, cx| remove_keys(&remove_id, cx)),
            )
        })
        .when(row.customized, |actions| {
            actions.child(
                Button::new(SharedString::from(format!("reset-{}", row.id)))
                    .ghost()
                    .xsmall()
                    .icon(Icon::empty().path("icons/rotate-ccw.svg"))
                    .tooltip("Reset to default")
                    .on_click(move |_, _, cx| reset_keys(&reset_id, cx)),
            )
        })
}
