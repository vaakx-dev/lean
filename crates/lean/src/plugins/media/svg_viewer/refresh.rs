use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use gpui_kit::*;

use super::SvgViewer;
use super::aspect::aspect_ratio;

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
                    cx.background_spawn(async move { rasterize(&renderer, &String::from(&text)) })
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

fn rasterize(renderer: &SvgRenderer, text: &str) -> Result<Arc<RenderImage>> {
    let svg = renderer.parse_svg(text.as_bytes())?;
    if let Some(ratio) = aspect_ratio(text) {
        return render_at_width(renderer, &svg, fitted_width(ratio));
    }
    let image = render_at_width(renderer, &svg, PREVIEW_PIXELS)?;
    let size = image.size(0);
    if size.height.0 <= PREVIEW_PIXELS {
        return Ok(image);
    }
    let ratio = size.height.0 as f32 / size.width.0 as f32;
    render_at_width(renderer, &svg, fitted_width(ratio))
}

fn fitted_width(ratio: f32) -> i32 {
    ((PREVIEW_PIXELS as f32 / ratio.max(1.)) as i32).max(1)
}

fn render_at_width(
    renderer: &SvgRenderer,
    svg: &ParsedSvg,
    width: i32,
) -> Result<Arc<RenderImage>> {
    let target = SvgSize::Size(size(DevicePixels(width), DevicePixels(width)));
    Ok(renderer.render_parsed(svg, target)?)
}
