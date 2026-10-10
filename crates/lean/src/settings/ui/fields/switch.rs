use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::{IntoElement as _, SharedString};

use super::field::Field;
use crate::settings::Setting;

pub(crate) fn switch(
    setting: &Setting<bool>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
) -> Field {
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, _, cx| {
        let setting = control_setting.clone();
        Switch::new(SharedString::from(format!("switch-{}", setting.key)))
            .checked(setting.get(cx))
            .disabled(options.is_disabled())
            .with_size(options.size())
            .on_click(move |checked: &bool, _, cx| setting.set(*checked, cx))
            .into_any_element()
    })
}
