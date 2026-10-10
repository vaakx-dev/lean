use gpui_kit::component::{Colorize as _, ThemeConfigColors};
use gpui_kit::{Hsla, SharedString};

const OPAQUE: u8 = 0xff;
const TINT: u8 = 0x1f;
const SOFT: u8 = 0x24;
const SELECTION: u8 = 0x38;
const BORDER: u8 = 0x66;
const HOVER_LIGHTEN: f32 = 0.1;
const LINK_HOVER_LIGHTEN: f32 = 0.15;
const ACTIVE_DARKEN: f32 = 0.2;

pub(super) fn apply_accent(colors: &mut ThemeConfigColors, accent: &str) {
    let Ok(color) = Hsla::parse_hex(accent) else {
        log::warn!("ignoring the accent colour {accent}: it is not a #rrggbb colour");
        return;
    };
    let color = Hsla { a: 1., ..color };
    let solid = hex(color, OPAQUE);
    let hover = hex(color.lighten(HOVER_LIGHTEN), OPAQUE);
    let active = hex(color.darken(ACTIVE_DARKEN), OPAQUE);
    let on_accent = contrasting(color, colors);

    colors.ring = Some(solid.clone());
    colors.caret = Some(solid.clone());
    colors.drag_border = Some(solid.clone());
    colors.link = Some(solid.clone());
    colors.primary = Some(solid.clone());
    colors.sidebar_primary = Some(solid);
    colors.primary_hover = Some(hover);
    colors.primary_active = Some(active.clone());
    colors.link_hover = Some(hex(color.lighten(LINK_HOVER_LIGHTEN), OPAQUE));
    colors.link_active = Some(active);
    colors.accent = Some(hex(color, TINT));
    colors.sidebar_accent = Some(hex(color, TINT));
    colors.drop_target = Some(hex(color, SOFT));
    colors.list_active = Some(hex(color, SOFT));
    colors.selection = Some(hex(color, SELECTION));
    colors.list_active_border = Some(hex(color, BORDER));
    if let Some(on_accent) = on_accent {
        colors.primary_foreground = Some(on_accent.clone());
        colors.sidebar_primary_foreground = Some(on_accent);
    }
}

fn contrasting(accent: Hsla, colors: &ThemeConfigColors) -> Option<SharedString> {
    let candidates = [colors.background.clone(), colors.foreground.clone()];
    candidates
        .into_iter()
        .flatten()
        .filter_map(|value| {
            let color = Hsla::parse_hex(&value).ok()?;
            Some(((color.l - accent.l).abs(), value))
        })
        .max_by(|(left, _), (right, _)| left.total_cmp(right))
        .map(|(_, value)| value)
}

fn hex(color: Hsla, alpha: u8) -> SharedString {
    let rgb = color.to_rgb();
    format!(
        "#{:02x}{:02x}{:02x}{alpha:02x}",
        channel(rgb.r),
        channel(rgb.g),
        channel(rgb.b)
    )
    .into()
}

fn channel(value: f32) -> u8 {
    (value.clamp(0., 1.) * 255.).round() as u8
}
