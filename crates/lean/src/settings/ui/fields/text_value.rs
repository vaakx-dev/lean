use gpui_kit::SharedString;

use crate::settings::SettingValue;

pub(crate) trait TextValue: SettingValue {
    fn to_text(&self) -> SharedString;
    fn from_text(text: &str) -> Self;
}

impl TextValue for String {
    fn to_text(&self) -> SharedString {
        self.clone().into()
    }

    fn from_text(text: &str) -> Self {
        text.trim().to_owned()
    }
}

impl TextValue for Option<String> {
    fn to_text(&self) -> SharedString {
        self.clone().unwrap_or_default().into()
    }

    fn from_text(text: &str) -> Self {
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_owned())
    }
}

pub(super) fn trimmed(text: &str) -> SharedString {
    SharedString::from(text.trim().to_owned())
}
