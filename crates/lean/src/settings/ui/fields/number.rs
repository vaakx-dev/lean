use gpui_kit::component::input::NumberInput;
use gpui_kit::component::setting::RenderOptions;
use gpui_kit::component::{ActiveTheme as _, Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::draft::{Commit, DraftInput};
use super::field::Field;
use super::number_value::{NumberValue, format_number, parse_number};
use crate::settings::Setting;

#[derive(Clone)]
pub(crate) struct Number {
    min: f64,
    max: f64,
    step: f64,
    unit: Option<SharedString>,
    placeholder: Option<SharedString>,
}

impl Number {
    pub(crate) fn range(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            step: 1.,
            unit: None,
            placeholder: None,
        }
    }

    pub(crate) fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub(crate) fn unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    pub(crate) fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
}

pub(crate) fn number<T: NumberValue>(
    setting: &Setting<T>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    number: Number,
) -> Field {
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, window, cx| {
        render_number(&control_setting, &number, options, window, cx)
    })
    .control_width(rems(8.))
}

fn render_number<T: NumberValue>(
    setting: &Setting<T>,
    number: &Number,
    options: &RenderOptions,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let draft = DraftInput {
        key: setting.key,
        value: format_number(setting.get(cx).to_number()).into(),
        normalize: normalize_number,
    };
    let configured = number.clone();
    let committed = number.clone();
    let setting = setting.clone();
    let input = draft.build(
        window,
        cx,
        move |input| {
            let input = input
                .step(configured.step)
                .min(configured.min)
                .max(configured.max);
            match configured.placeholder {
                Some(placeholder) => input.placeholder(placeholder),
                None => input,
            }
        },
        move |text, kind, cx| {
            if let Some(value) = parse_commit::<T>(text, kind, &committed) {
                setting.set(value, cx);
            }
        },
    );
    let muted = cx.theme().muted_foreground;
    NumberInput::new(&input)
        .disabled(options.is_disabled())
        .with_size(options.size())
        .when_some(number.unit.clone(), |input, unit| {
            input.suffix(div().text_xs().text_color(muted).child(unit))
        })
        .w_full()
        .into_any_element()
}

fn parse_commit<T: NumberValue>(text: &str, kind: Commit, number: &Number) -> Option<T> {
    let parsed = parse_number(text).ok()?;
    let value = match parsed {
        Some(value) if (number.min..=number.max).contains(&value) => Some(value),
        Some(_) if kind == Commit::Typing => return None,
        Some(value) => Some(value.clamp(number.min, number.max)),
        None => None,
    };
    T::from_number(value)
}

fn normalize_number(text: &str) -> SharedString {
    match parse_number(text) {
        Ok(number) => format_number(number).into(),
        Err(()) => SharedString::from(text.to_owned()),
    }
}
