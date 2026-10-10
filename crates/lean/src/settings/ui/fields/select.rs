use std::rc::Rc;

use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::field::Field;
use super::is_stacked;
use crate::settings::{Setting, SettingValue};

pub(crate) fn select<T: SettingValue>(
    setting: &Setting<T>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    choices: impl IntoIterator<Item = (T, impl Into<SharedString>)>,
) -> Field {
    let choices: Rc<[(T, SharedString)]> = choices
        .into_iter()
        .map(|(value, label)| (value, label.into()))
        .collect();
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, _, cx| {
        let current = control_setting.get(cx);
        let label = choices
            .iter()
            .find(|(value, _)| *value == current)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Custom".into());
        let choices = choices.clone();
        let setting = control_setting.clone();
        Button::new(SharedString::from(format!("select-{}", setting.key)))
            .label(label)
            .dropdown_caret(true)
            .outline()
            .disabled(options.is_disabled())
            .with_size(options.size())
            .when(is_stacked(options), |button| button.w_full())
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
                choices
                    .iter()
                    .fold(menu, |menu, (value, label)| {
                        let setting = setting.clone();
                        let value = value.clone();
                        menu.item(
                            PopupMenuItem::new(label.clone())
                                .checked(value == current)
                                .on_click(move |_, _, cx| setting.set(value.clone(), cx)),
                        )
                    })
                    .scrollable(true)
            })
            .into_any_element()
    })
}
