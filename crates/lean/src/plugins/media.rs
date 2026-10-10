mod checkerboard;
mod formats;
mod image_viewer;
mod picture;
pub(crate) mod settings;
mod svg_viewer;

use gpui_kit::App;

use super::{Plugin, PluginContext};

pub(super) const PLUGIN: Plugin = Plugin {
    id: "media",
    name: "Images & SVG",
    description: "View images, and preview SVG files next to their code.",
    activate,
};

fn activate(context: &mut PluginContext, cx: &mut App) {
    image_viewer::bind_keys(cx);
    context.register_viewer(image_viewer::IMAGE_VIEWER);
    context.register_viewer(svg_viewer::SVG_VIEWER);
}
