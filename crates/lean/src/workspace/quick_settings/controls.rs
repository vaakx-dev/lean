use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Icon, Sizable as _, h_flex};
use gpui_kit::*;

use crate::settings::Setting;
use crate::theme::settings::{THEME, UI_FONT_SIZE, UI_FONT_SIZE_RANGE};
use crate::theme::theme_names;

const SELECT_WIDTH: Pixels = px(132.);
const STEPPER_HEIGHT: Pixels = px(26.);
const STEPPER_VALUE_WIDTH: Pixels = px(34.);

pub(super) fn theme_select(menu_open: Entity<bool>, cx: &App) -> impl IntoElement {
    let current = SharedString::from(THEME.get(cx));
    let themes = theme_names(cx);
    Button::new("quick-settings-theme")
        .label(current.clone())
        .dropdown_caret(true)
        .outline()
        .small()
        .w(SELECT_WIDTH)
        .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
            themes
                .iter()
                .fold(menu, |menu, name| {
                    let value = name.to_string();
                    menu.item(
                        PopupMenuItem::new(name.clone())
                            .checked(*name == current)
                            .on_click(move |_, _, cx| THEME.set(value.clone(), cx)),
                    )
                })
                .scrollable(true)
        })
        .on_open_change(move |open, _, cx| {
            menu_open.update(cx, |state, _| *state = *open);
        })
}

pub(super) fn font_size_stepper(cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let size = UI_FONT_SIZE.get(cx);
    let (min, max) = UI_FONT_SIZE_RANGE;
    h_flex()
        .h(STEPPER_HEIGHT)
        .border_1()
        .border_color(theme.input)
        .rounded(theme.radius)
        .child(step_button(
            "quick-settings-font-smaller",
            "icons/minus.svg",
            size > min,
            (size.round() - 1.).clamp(min, max),
        ))
        .child(
            div()
                .h_full()
                .min_w(STEPPER_VALUE_WIDTH)
                .flex()
                .items_center()
                .justify_center()
                .border_x_1()
                .border_color(theme.border)
                .text_sm()
                .child(format!("{size}")),
        )
        .child(step_button(
            "quick-settings-font-larger",
            "icons/plus.svg",
            size < max,
            (size.round() + 1.).clamp(min, max),
        ))
}

pub(super) fn setting_switch(id: &'static str, setting: Setting<bool>, cx: &App) -> Switch {
    Switch::new(id)
        .checked(setting.get(cx))
        .small()
        .on_click(move |checked: &bool, window, cx| {
            setting.set(*checked, cx);
            window.refresh();
        })
}

fn step_button(id: &'static str, icon: &'static str, enabled: bool, value: f32) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::empty().path(icon))
        .disabled(!enabled)
        .on_click(move |_, window, cx| {
            UI_FONT_SIZE.set(value, cx);
            window.refresh();
        })
}
