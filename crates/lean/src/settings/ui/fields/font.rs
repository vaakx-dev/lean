use gpui_kit::component::select::{SearchableVec, Select, SelectEvent, SelectState};
use gpui_kit::component::{IndexPath, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::field::Field;
use super::is_stacked;
use super::text_value::TextValue;
use crate::settings::Setting;
use crate::theme::installed_font_names;

const FONT_WIDTH: Pixels = px(240.);

type FontList = SearchableVec<SharedString>;

struct FontPicker {
    select: Entity<SelectState<FontList>>,
    synced: SharedString,
    _subscription: Subscription,
}

pub(crate) fn font<T: TextValue>(
    setting: &Setting<T>,
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    placeholder: impl Into<SharedString>,
) -> Field {
    let placeholder = placeholder.into();
    let control_setting = setting.clone();
    Field::bound(setting, title, description, move |options, window, cx| {
        let select = font_select(&control_setting, window, cx);
        Select::new(&select)
            .placeholder(placeholder.clone())
            .search_placeholder("Search fonts")
            .cleanable(true)
            .disabled(options.is_disabled())
            .with_size(options.size())
            .map(|select| {
                if is_stacked(options) {
                    select.w_full()
                } else {
                    select.w(FONT_WIDTH)
                }
            })
            .into_any_element()
    })
}

fn font_select<T: TextValue>(
    setting: &Setting<T>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<SelectState<FontList>> {
    let value = setting.get(cx).to_text();
    let id = SharedString::from(format!("setting-font-{}", setting.key));
    let picker = window.use_keyed_state(id, cx, |window, cx| {
        FontPicker::new(setting.clone(), value.clone(), window, cx)
    });
    picker.update(cx, |picker, cx| picker.sync(value, window, cx));
    picker.read(cx).select.clone()
}

impl FontPicker {
    fn new<T: TextValue>(
        setting: Setting<T>,
        value: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let fonts = installed_font_names(cx);
        let selected = fonts
            .iter()
            .position(|font| *font == value)
            .map(|row| IndexPath::default().row(row));
        let list = SearchableVec::new(fonts.to_vec());
        let select = cx.new(|cx| SelectState::new(list, selected, window, cx).searchable(true));
        let subscription =
            cx.subscribe(
                &select,
                move |_, _, event: &SelectEvent<FontList>, cx| match event {
                    SelectEvent::Confirm(Some(font)) => setting.set(T::from_text(font), cx),
                    SelectEvent::Confirm(None) => setting.reset(cx),
                },
            );
        Self {
            select,
            synced: value,
            _subscription: subscription,
        }
    }

    fn sync(&mut self, value: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if value == self.synced {
            return;
        }
        self.synced = value.clone();
        self.select.update(cx, |select, cx| {
            select.set_selected_value(&value, window, cx)
        });
    }
}
