use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::Input;
use gpui_kit::*;

use super::draft::DraftInput;
use super::field::Field;
use crate::settings::Setting;

const SEPARATOR: char = ',';

pub(crate) fn list(
    setting: &Setting<Vec<String>>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    placeholder: impl Into<SharedString>,
) -> Field {
    let placeholder = placeholder.into();
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, window, cx| {
        let setting = control_setting.clone();
        let draft = DraftInput {
            key: setting.key,
            value: join(&setting.get(cx)),
            normalize: normalize_list,
        };
        let placeholder = placeholder.clone();
        let input = draft.build(
            window,
            cx,
            move |input| input.placeholder(placeholder),
            move |text, _, cx| setting.set(split(text), cx),
        );
        Input::new(&input)
            .disabled(options.is_disabled())
            .with_size(options.size())
            .w_full()
            .into_any_element()
    })
    .control_width(px(300.))
}

fn split(text: &str) -> Vec<String> {
    text.split(SEPARATOR)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn join(items: &[String]) -> SharedString {
    SharedString::from(items.join(", "))
}

fn normalize_list(text: &str) -> SharedString {
    join(&split(text))
}
