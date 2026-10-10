use gpui_kit::{DevicePixels, Pixels, Size, px};

pub(super) const PADDING: Pixels = px(16.);
pub(super) const ZOOM_STEP: f32 = 1.25;
const MIN_SCALE: f32 = 0.02;
const MAX_SCALE: f32 = 64.;
const WHEEL_SPEED: f32 = 0.004;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Zoom {
    Fit,
    Scale(f32),
}

impl Zoom {
    pub(super) fn scale(
        self,
        image: Size<DevicePixels>,
        viewport: Size<Pixels>,
        upscale: bool,
    ) -> Option<f32> {
        match self {
            Zoom::Scale(scale) => Some(scale),
            Zoom::Fit => fit_scale(image, viewport, upscale),
        }
    }
}

pub(super) fn clamp_scale(scale: f32) -> f32 {
    scale.clamp(MIN_SCALE, MAX_SCALE)
}

pub(super) fn wheel_factor(delta: Pixels) -> f32 {
    (delta.as_f32() * WHEEL_SPEED).exp()
}

pub(super) fn displayed_length(length: DevicePixels, scale: f32) -> Pixels {
    px(length.0 as f32 * scale)
}

pub(super) fn rescaled_offset(
    viewport: f32,
    anchor: f32,
    offset: f32,
    old_length: f32,
    new_length: f32,
) -> f32 {
    if old_length <= 0. {
        return 0.;
    }
    let image_point = anchor - offset - image_origin(viewport, old_length);
    let offset =
        anchor - image_origin(viewport, new_length) - image_point * new_length / old_length;
    let content = (new_length + 2. * PADDING.as_f32()).max(viewport);
    offset.clamp((viewport - content).min(0.), 0.)
}

fn image_origin(viewport: f32, length: f32) -> f32 {
    ((length + 2. * PADDING.as_f32()).max(viewport) - length) / 2.
}

fn fit_scale(image: Size<DevicePixels>, viewport: Size<Pixels>, upscale: bool) -> Option<f32> {
    let width = (viewport.width - PADDING * 2.).as_f32();
    let height = (viewport.height - PADDING * 2.).as_f32();
    if width <= 0. || height <= 0. || image.width.0 <= 0 || image.height.0 <= 0 {
        return None;
    }
    let scale = (width / image.width.0 as f32).min(height / image.height.0 as f32);
    Some(if upscale { scale } else { scale.min(1.) })
}
