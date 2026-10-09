use gpui_kit::App;

use crate::plugins::AnyFileView;

pub(super) type SvgSource = Box<dyn FnOnce() -> Vec<u8> + Send>;

pub(super) fn svg_source(code: &AnyFileView, cx: &App) -> Option<SvgSource> {
    let text = code.text(cx)?;
    Some(Box::new(move || String::from(&text).into_bytes()))
}
