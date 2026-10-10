use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::input::Input;
use gpui_kit::component::setting::{SettingGroup, SettingItem};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::capture::{TableState, Target};
use super::keys::sequence_label;
use super::row::{ACTIONS_WIDTH, CONTEXT_WIDTH, KEYS_WIDTH, SOURCE_WIDTH, render_row};
use crate::app::keymap::{KEYMAP, Row, conflicts, rows};

const STATE_KEY: &str = "keybindings-table";

pub(super) fn shortcuts_group(cx: &App) -> SettingGroup {
    SettingGroup::new().variant(GroupBoxVariant::Normal).item(
        SettingItem::render(|_, window, cx| render_table(window, cx))
            .keywords(search_keywords(cx))
            .on_reset(|cx| KEYMAP.is_modified(cx), |_, cx| KEYMAP.reset(cx)),
    )
}

fn search_keywords(cx: &App) -> Vec<SharedString> {
    let mut keywords: Vec<SharedString> = ["keybindings", "shortcuts", "keymap", "hotkeys"]
        .into_iter()
        .map(SharedString::from)
        .collect();
    for row in rows(cx) {
        keywords.push(row.title.clone());
        keywords.extend(row.keys.iter().cloned());
    }
    keywords
}

fn render_table(window: &mut Window, cx: &mut App) -> Div {
    let state = window.use_keyed_state(STATE_KEY, cx, TableState::new);
    let cx: &App = cx;
    let query = state.read(cx).search.read(cx).value().trim().to_lowercase();
    let all_rows = rows(cx);
    let row_conflicts: Vec<Vec<SharedString>> = all_rows
        .iter()
        .map(|row| conflicts(row, &all_rows))
        .collect();
    let visible: Vec<usize> = (0..all_rows.len())
        .filter(|&index| matches_query(&all_rows[index], &query))
        .collect();
    v_flex()
        .w_full()
        .gap_3()
        .child(toolbar(&state, cx))
        .child(
            v_flex()
                .w_full()
                .overflow_hidden()
                .rounded(cx.theme().radius_lg)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().table)
                .child(header(cx))
                .children(
                    visible.iter().map(|&index| {
                        render_row(&all_rows[index], &row_conflicts[index], &state, cx)
                    }),
                )
                .when(visible.is_empty(), |table| {
                    table.child(empty_state(&query, cx))
                }),
        )
        .child(summary(&all_rows, &row_conflicts, cx))
}

fn toolbar(state: &Entity<TableState>, cx: &App) -> Div {
    let table = state.read(cx);
    let recording_keys = table.is_recording(&Target::Search);
    let search = table.search.clone();
    let focus_handle = table.focus_handle.clone();
    let state = state.clone();
    h_flex()
        .w_full()
        .gap_2()
        .child(
            Input::new(&search).flex_1().small().cleanable(true).prefix(
                Icon::empty()
                    .path("icons/search.svg")
                    .size_4()
                    .text_color(cx.theme().muted_foreground),
            ),
        )
        .child(
            div()
                .when(recording_keys, |slot| slot.track_focus(&focus_handle))
                .child(
                    Button::new("record-search-keys")
                        .outline()
                        .small()
                        .icon(Icon::empty().path("icons/keyboard.svg"))
                        .label(if recording_keys {
                            "Press keys…"
                        } else {
                            "Record keys"
                        })
                        .toggled(recording_keys)
                        .on_click(move |_, window, cx| {
                            state.update(cx, |state, cx| state.toggle(Target::Search, window, cx));
                        }),
                ),
        )
}

fn header(cx: &App) -> Div {
    let theme = cx.theme();
    h_flex()
        .w_full()
        .h(px(30.))
        .px_3()
        .gap_3()
        .bg(theme.table_head)
        .text_xs()
        .font_semibold()
        .text_color(theme.table_head_foreground)
        .child(div().flex_1().min_w_0().child("Command"))
        .child(div().w(KEYS_WIDTH).flex_shrink_0().child("Keybinding"))
        .child(div().w(CONTEXT_WIDTH).flex_shrink_0().child("Context"))
        .child(div().w(SOURCE_WIDTH).flex_shrink_0().child("Source"))
        .child(div().w(ACTIONS_WIDTH).flex_shrink_0())
}

fn empty_state(query: &str, cx: &App) -> Div {
    div()
        .w_full()
        .py_6()
        .border_t_1()
        .border_color(cx.theme().table_row_border)
        .text_center()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(format!("No commands match “{query}”"))
}

fn summary(rows: &[Row], conflicts: &[Vec<SharedString>], cx: &App) -> Div {
    let theme = cx.theme();
    let changed = rows.iter().filter(|row| row.customized).count();
    let conflicting = conflicts.iter().filter(|titles| !titles.is_empty()).count();
    h_flex()
        .w_full()
        .gap_2()
        .justify_between()
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(
            h_flex()
                .gap_2()
                .child(match changed {
                    0 => "No changes from the defaults".to_owned(),
                    1 => "1 changed from default".to_owned(),
                    count => format!("{count} changed from default"),
                })
                .when(conflicting > 0, |line| {
                    line.child("·").child(div().text_color(theme.warning).child(
                        match conflicting {
                            1 => "1 command in conflict".to_owned(),
                            count => format!("{count} commands in conflict"),
                        },
                    ))
                }),
        )
        .when(changed > 0, |line| {
            line.child(
                Button::new("reset-all-keybindings")
                    .ghost()
                    .xsmall()
                    .icon(Icon::empty().path("icons/rotate-ccw.svg"))
                    .label("Reset all")
                    .on_click(|_, _, cx| KEYMAP.reset(cx)),
            )
        })
}

fn matches_query(row: &Row, query: &str) -> bool {
    query.is_empty()
        || row.title.to_lowercase().contains(query)
        || row.id.to_lowercase().contains(query)
        || row.context.to_lowercase().contains(query)
        || row.keys.iter().any(|sequence| {
            sequence.to_lowercase().contains(query)
                || sequence_label(sequence).to_lowercase().contains(query)
        })
}
