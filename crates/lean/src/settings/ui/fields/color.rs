use gpui_kit::component::color_picker::{ColorPickerEvent, ColorPickerState, ColorSelect};
use gpui_kit::component::{Colorize as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::field::Field;
use super::is_stacked;
use crate::settings::Setting;

const COLOR_WIDTH: Pixels = px(160.);

struct ColorField {
    picker: Entity<ColorPickerState>,
    synced: String,
    _subscription: Subscription,
}

pub(crate) fn color(
    setting: &Setting<String>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
) -> Field {
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, window, cx| {
        let picker = color_picker(&control_setting, window, cx);
        ColorSelect::new(&picker)
            .with_size(options.size())
            .map(|select| {
                if is_stacked(options) {
                    select.w_full()
                } else {
                    select.w(COLOR_WIDTH)
                }
            })
            .into_any_element()
    })
}

fn color_picker(
    setting: &Setting<String>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ColorPickerState> {
    let value = setting.get(cx);
    let id = SharedString::from(format!("setting-color-{}", setting.key));
    let field = window.use_keyed_state(id, cx, |window, cx| {
        ColorField::new(setting.clone(), value.clone(), window, cx)
    });
    field.update(cx, |field, cx| field.sync(value, window, cx));
    field.read(cx).picker.clone()
}

impl ColorField {
    fn new(
        setting: Setting<String>,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let picker = cx.new(|cx| {
            let picker = ColorPickerState::new(window, cx);
            match parse_color(&value) {
                Some(color) => picker.default_value(color),
                None => picker,
            }
        });
        let subscription =
            cx.subscribe(
                &picker,
                move |_, _, event: &ColorPickerEvent, cx| match event {
                    ColorPickerEvent::Change(Some(color)) => {
                        setting.set(color.to_hex().to_lowercase(), cx)
                    }
                    ColorPickerEvent::Change(None) => setting.reset(cx),
                },
            );
        Self {
            picker,
            synced: value,
            _subscription: subscription,
        }
    }

    fn sync(&mut self, value: String, window: &mut Window, cx: &mut Context<Self>) {
        if value == self.synced {
            return;
        }
        let color = parse_color(&value);
        self.synced = value;
        self.picker.update(cx, |picker, cx| match color {
            Some(color) => picker.set_value(color, window, cx),
            None => picker.clear_value(window, cx),
        });
    }
}

fn parse_color(value: &str) -> Option<Hsla> {
    Hsla::parse_hex(value).ok()
}
