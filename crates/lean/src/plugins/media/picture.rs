use std::sync::Arc;

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::checkerboard::checkerboard;
use super::settings::{BACKGROUND, ImageBackground};

#[derive(Clone, Copy)]
pub(super) enum Fit {
    Contain,
    ScaleDown,
}

impl Fit {
    pub(super) fn object_fit(self) -> ObjectFit {
        match self {
            Fit::Contain => ObjectFit::Contain,
            Fit::ScaleDown => ObjectFit::ScaleDown,
        }
    }
}

pub(super) fn picture(
    id: impl Into<ElementId>,
    image: &Arc<RenderImage>,
    fit: Fit,
    cx: &App,
) -> Div {
    let checkered = BACKGROUND.get(cx) == ImageBackground::Checkerboard;
    div()
        .relative()
        .when(checkered, |picture| {
            picture.child(checkerboard(image.size(0), fit, cx))
        })
        .child(
            img(image.clone())
                .id(id)
                .absolute()
                .size_full()
                .object_fit(fit.object_fit()),
        )
}
