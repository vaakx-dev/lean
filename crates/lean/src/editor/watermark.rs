use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::{ActiveTheme as _, h_flex, v_flex};
use gpui_kit::*;

use crate::app::actions::{OpenFolder, OpenSettings, ToggleSidebar, ToggleTerminal};

pub(super) fn render_watermark(window: &Window, cx: &App) -> Div {
    let shortcuts: [(&'static str, &dyn Action); 4] = [
        ("Open Folder", &OpenFolder),
        ("Toggle Sidebar", &ToggleSidebar),
        ("Toggle Terminal", &ToggleTerminal),
        ("Open Settings", &OpenSettings),
    ];
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
            v_flex().w(px(340.)).gap_3().children(
                shortcuts
                    .into_iter()
                    .filter_map(|(label, action)| shortcut(label, action, window, cx)),
            ),
        )
}

fn shortcut(label: &'static str, action: &dyn Action, window: &Window, cx: &App) -> Option<Div> {
    let key = Kbd::global_binding_for_action(action, window)?;
    Some(
        h_flex()
            .w_full()
            .justify_between()
            .gap_8()
            .child(div().text_color(cx.theme().muted_foreground).child(label))
            .child(key.outline()),
    )
}
