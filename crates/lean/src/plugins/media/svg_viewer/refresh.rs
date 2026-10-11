use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use gpui_kit::*;

use super::SvgViewer;

const PREVIEW_PIXELS: i32 = 1536;

impl SvgViewer {
    pub(super) fn schedule_refresh(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let renderer = cx.svg_renderer();
        self._refresh = cx.spawn(async move |viewer, cx| {
            cx.background_executor().timer(delay).await;
            let Ok(text) = viewer.update(cx, |viewer, cx| viewer.code.text(cx)) else {
                return;
            };
            let rendered = match text {
                Some(text) => {
                    cx.background_spawn(async move {
                        rasterize(&renderer, String::from(&text).as_bytes())
                    })
                    .await
                }
                None => Err(anyhow!("The file isn't valid UTF-8 text.")),
            };
            viewer
                .update(cx, |viewer, cx| {
                    viewer.preview.show(rendered, cx);
                    cx.notify();
                })
                .ok();
        });
    }
}

fn rasterize(renderer: &SvgRenderer, bytes: &[u8]) -> Result<Arc<RenderImage>> {
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
