use gpui_kit::component::{ActiveTheme as _, Theme, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

pub(super) fn render_watermark(cx: &App) -> Div {
    let theme = cx.theme();
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_12()
        .child(
            img("images/lean.png")
                .size(px(280.))
                .opacity(0.35)
                .with_fallback(|| div().into_any_element()),
        )
        .child(
            v_flex()
                .w(px(340.))
                .gap_3()
                .child(shortcut("Open Folder", &["Ctrl", "O"], theme))
                .child(shortcut("Toggle Sidebar", &["Ctrl", "B"], theme))
                .child(shortcut("Toggle Terminal", &["Ctrl", "`"], theme)),
        )
}

fn shortcut(label: &'static str, keys: &'static [&'static str], theme: &Theme) -> Div {
    h_flex()
        .w_full()
        .justify_between()
        .gap_8()
        .child(div().text_color(theme.muted_foreground).child(label))
        .child(
            h_flex().gap_1().children(
                keys.iter()
                    .copied()
                    .enumerate()
                    .map(|(index, key)| key_cap(index, key, theme)),
            ),
        )
}

fn key_cap(index: usize, key: &'static str, theme: &Theme) -> Div {
    h_flex()
        .gap_1()
        .when(index > 0, |keys| {
            keys.child(div().text_color(theme.muted_foreground).child("+"))
        })
        .child(
            div()
                .px_1p5()
                .py_0p5()
                .text_xs()
                .rounded(px(4.))
                .border_1()
                .border_color(theme.border)
                .bg(theme.muted)
                .text_color(theme.foreground)
                .child(key),
        )
}
