use gpui_kit::component::input::InputState;
use gpui_kit::*;

use super::draft::DraftInput;
use super::text_value::{TextValue, trimmed};
use crate::settings::Setting;

pub(super) fn text_input<T: TextValue>(
    setting: &Setting<T>,
    placeholder: SharedString,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    let draft = DraftInput {
        key: setting.key,
        value: setting.get(cx).to_text(),
        normalize: trimmed,
    };
    let setting = setting.clone();
    draft.build(
        window,
        cx,
        move |input| input.placeholder(placeholder),
        move |text, _, cx| setting.set(T::from_text(text), cx),
    )
}
