use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use super::ImageViewer;
use crate::files::format_size;
use crate::plugins::media::formats::format_name;
use crate::workspace::status_item;

impl ImageViewer {
    pub(super) fn render_status_items(&self, cx: &App) -> Vec<AnyElement> {
        let theme = cx.theme();
        let dimensions = self.image_size().map(|size| {
            status_item("image-dimensions", theme)
                .child(format!("{} × {}", size.width.0, size.height.0))
                .into_any_element()
        });
        dimensions
            .into_iter()
            .chain([
                status_item("file-size", theme)
                    .child(format_size(self.image.bytes().len()))
                    .into_any_element(),
                status_item("image-format", theme)
                    .child(format_name(self.image.format()))
                    .into_any_element(),
            ])
            .collect()
    }
}
