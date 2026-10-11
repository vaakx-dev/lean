use crate::settings::SettingValue;

pub(crate) trait NumberValue: SettingValue {
    fn to_number(&self) -> Option<f64>;
    fn from_number(number: Option<f64>) -> Option<Self>;
}

macro_rules! integer_number_values {
    ($($kind:ty),*) => {
        $(impl NumberValue for $kind {
            fn to_number(&self) -> Option<f64> {
                Some(*self as f64)
            }

            fn from_number(number: Option<f64>) -> Option<Self> {
                number.map(|number| number.round() as $kind)
            }
        })*
    };
}

integer_number_values!(u32, u64, usize);

impl NumberValue for f32 {
    fn to_number(&self) -> Option<f64> {
        self.to_string().parse().ok()
    }

    fn from_number(number: Option<f64>) -> Option<Self> {
        number.map(|number| number as f32)
    }
}

impl<T: NumberValue> NumberValue for Option<T> {
    fn to_number(&self) -> Option<f64> {
        self.as_ref().and_then(T::to_number)
    }

    fn from_number(number: Option<f64>) -> Option<Self> {
        match number {
            Some(number) => T::from_number(Some(number)).map(Some),
            None => Some(None),
        }
    }
}

pub(super) fn format_number(number: Option<f64>) -> String {
    number.map(|number| number.to_string()).unwrap_or_default()
}

pub(super) fn parse_number(text: &str) -> Result<Option<f64>, ()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    text.parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .map(Some)
        .ok_or(())
}
