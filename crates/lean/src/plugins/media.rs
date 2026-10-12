mod checkerboard;
mod empty_state;
mod formats;
mod image_viewer;
mod picture;
pub(crate) mod settings;
mod svg_viewer;

use gpui_kit::App;

use super::{Plugin, Registry};

pub(super) const PLUGIN: Plugin = Plugin {
    id: "media",
    name: "Images & SVG",
    description: "View images, and preview SVG files next to their code.",
    activate,
};

fn activate(registry: &mut Registry, cx: &mut App) {
    image_viewer::bind_keys(cx);
    registry.register_viewer(image_viewer::IMAGE_VIEWER);
    registry.register_viewer(svg_viewer::SVG_VIEWER);
}
