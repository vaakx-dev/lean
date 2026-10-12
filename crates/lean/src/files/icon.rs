use gpui_kit::component::Icon;
use gpui_kit::{Styled, rgb};

use super::file_type::file_type;

pub(crate) fn file_icon(name: &str) -> Icon {
    let (path, color) = file_type(name).icon;
    Icon::empty().path(path).size_4().text_color(rgb(color))
}
