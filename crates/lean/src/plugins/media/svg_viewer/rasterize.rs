use std::sync::Arc;

use anyhow::Result;
use gpui_kit::{DevicePixels, ParsedSvg, RenderImage, SvgRenderer, SvgSize, size};

const PREVIEW_PIXELS: i32 = 1536;

pub(super) fn rasterize(renderer: &SvgRenderer, bytes: &[u8]) -> Result<Arc<RenderImage>> {
    let svg = renderer.parse_svg(bytes)?;
    let image = render_at_width(renderer, &svg, PREVIEW_PIXELS)?;
    let height = image.size(0).height.0;
    if height <= PREVIEW_PIXELS {
        return Ok(image);
    }
    let width = i64::from(PREVIEW_PIXELS) * i64::from(PREVIEW_PIXELS) / i64::from(height);
    let width = i32::try_from(width).unwrap_or(PREVIEW_PIXELS).max(1);
    render_at_width(renderer, &svg, width)
}

fn render_at_width(
    renderer: &SvgRenderer,
    svg: &ParsedSvg,
    width: i32,
) -> Result<Arc<RenderImage>> {
    let target = SvgSize::Size(size(DevicePixels(width), DevicePixels(width)));
    Ok(renderer.render_parsed(svg, target)?)
}
