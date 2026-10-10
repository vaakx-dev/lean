mod color;
mod draft;
mod field;
mod font;
mod list;
mod number;
mod number_value;
mod path;
mod row;
mod select;
mod switch;
mod text;
mod text_value;

use gpui_kit::Axis;
use gpui_kit::component::setting::RenderOptions;

pub(crate) use color::color;
pub(crate) use field::{Field, group};
pub(crate) use font::font;
pub(crate) use list::list;
pub(crate) use number::{Number, number};
pub(crate) use path::{PathKind, path};
pub(crate) use select::select;
pub(crate) use switch::switch;

fn is_stacked(options: &RenderOptions) -> bool {
    options.layout() == Axis::Vertical
}
