use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::{ActiveTheme as _, h_flex};
use gpui_kit::*;

pub(super) fn sequence_chips(sequence: &str, cx: &App) -> Div {
    h_flex().gap_1().flex_wrap().children(
        sequence
            .split_whitespace()
            .map(|keystroke| keystroke_chip(keystroke, cx)),
    )
}

pub(super) fn keystroke_chips(keystrokes: &[Keystroke]) -> Div {
    h_flex().gap_1().children(
        keystrokes
            .iter()
            .map(|keystroke| Kbd::new(keystroke.clone()).outline().into_any_element()),
    )
}

pub(super) fn sequence_label(sequence: &str) -> String {
    sequence
        .split_whitespace()
        .map(|keystroke| match Keystroke::parse(keystroke) {
            Ok(keystroke) => Kbd::format(&keystroke),
            Err(_) => keystroke.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn escape_chip() -> Kbd {
    Kbd::new(Keystroke {
        modifiers: Modifiers::none(),
        key: "escape".into(),
        key_char: None,
    })
}

fn keystroke_chip(keystroke: &str, cx: &App) -> AnyElement {
    match Keystroke::parse(keystroke) {
        Ok(keystroke) => Kbd::new(keystroke).outline().into_any_element(),
        Err(_) => div()
            .text_xs()
            .text_color(cx.theme().danger)
            .child(keystroke.to_owned())
            .into_any_element(),
    }
}
