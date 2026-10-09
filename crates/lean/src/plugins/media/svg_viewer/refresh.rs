use std::time::Duration;

use anyhow::anyhow;
use gpui_kit::*;

use super::SvgViewer;
use super::rasterize::rasterize;
use super::source::svg_source;

impl SvgViewer {
    pub(super) fn schedule_refresh(&mut self, delay: Duration, cx: &mut Context<Self>) {
        let renderer = cx.svg_renderer();
        self._refresh = cx.spawn(async move |viewer, cx| {
            cx.background_executor().timer(delay).await;
            let Ok(source) = viewer.update(cx, |viewer, cx| svg_source(&viewer.code, cx)) else {
                return;
            };
            let rendered = match source {
                Some(source) => {
                    cx.background_spawn(async move { rasterize(&renderer, &source()) })
                        .await
                }
                None => Err(anyhow!("The file isn't valid UTF-8 text.")),
            };
            let update = viewer.update(cx, |viewer, cx| {
                viewer.preview.show(rendered, cx);
                cx.notify();
            });
            if let Err(error) = update {
                log::debug!("svg viewer released before its preview finished: {error:#}");
            }
        });
    }
}
