use gpui_kit::*;

use super::ImageViewer;

actions!(image_viewer, [ZoomIn, ZoomOut, ZoomToFit, ActualSize]);

pub(super) const KEY_CONTEXT: &str = "ImageViewer";

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-=", ZoomIn, Some(KEY_CONTEXT)),
        KeyBinding::new("secondary-+", ZoomIn, Some(KEY_CONTEXT)),
        KeyBinding::new("secondary--", ZoomOut, Some(KEY_CONTEXT)),
        KeyBinding::new("secondary-0", ZoomToFit, Some(KEY_CONTEXT)),
        KeyBinding::new("secondary-1", ActualSize, Some(KEY_CONTEXT)),
    ]);
}

pub(super) fn bind_actions(element: Stateful<Div>, cx: &Context<ImageViewer>) -> Stateful<Div> {
    element
        .on_action(cx.listener(|viewer, _: &ZoomIn, _, cx| viewer.zoom_in(cx)))
        .on_action(cx.listener(|viewer, _: &ZoomOut, _, cx| viewer.zoom_out(cx)))
        .on_action(cx.listener(|viewer, _: &ZoomToFit, _, cx| viewer.zoom_to_fit(cx)))
        .on_action(cx.listener(|viewer, _: &ActualSize, _, cx| viewer.zoom_to_actual_size(cx)))
}
